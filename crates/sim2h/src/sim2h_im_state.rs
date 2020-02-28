use crate::*;
use lib3h::rrdht_util::Location;
use rand::Rng;
use serde::{Serialize, Serializer};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::stream::StreamExt;

fn should_store(
    agent_loc: Location,
    entry_loc: Location,
    agent_count: u64,
    redundancy: u64,
) -> bool {
    if redundancy == 0 {
        return true;
    }
    naive_sharding::naive_sharding_should_store(agent_loc, entry_loc, agent_count, redundancy)
}

/// How often(ish) should we check/fetch aspects for each connected agent
/// note this value is randomized by ~50% each time (i.e. 0.75x to 1.25x)
const AGENT_FETCH_ASPECTS_INTERVAL_MS: u64 = 5000;

/// add an increment function to AtomicU64
/// returns the previous value after making sure it is upped by 1
trait AtomicInc {
    fn inc(&self) -> u64;
}

impl AtomicInc for AtomicU64 {
    fn inc(&self) -> u64 {
        self.fetch_add(1, Ordering::SeqCst)
    }
}

/// Append-Only-Log Entries for mutating the Sim2h Store
/// with a list of these, we should be able to reconstruct the store
/// even if they come out-of-order.
#[derive(Debug)]
enum AolEntry {
    // all we know is this agent MAY be connected (if con_incr is > cur)
    // - set connections entry to is_connected=true
    // - add entry to `uri_to_connection`
    // - clear `holding`?
    NewConnection {
        aol_idx: u64,
        space_hash: SpaceHash,
        agent_id: AgentId,
        uri: Lib3hUri,
    },

    // we will no longer rely on this agent/connection (if con_incr is > cur)
    // - mark connection as disconnected (tombstone)
    // - clear all `holding` aspects (to prepare for another connection
    // - remove the uri_to_connection entry
    // - if there are no more connections in the space, drop it too
    DropConnection {
        aol_idx: u64,
        space_hash: SpaceHash,
        agent_id: AgentId,
    },

    // we need to be able to drop all connections across spaces based on
    // the uri of the connected socket (i.e. in case of a socket read/write err)
    // (see DropConnection for drop workflow)
    DropConnectionByUri {
        aol_idx: u64,
        uri: Lib3hUri,
    },

    // if this agent is currently assumed connected (&& con_incr is > cur)
    // mark that they are likely `holding` these aspects
    AgentHoldsAspects {
        aol_idx: u64,
        space_hash: SpaceHash,
        agent_id: AgentId,
        entry_hash: EntryHash,
        aspects: im::HashSet<AspectHash>,
    },

    // our owner is ready to do a round of gossip
    // check to see if any of our agents are ready to check
    // for missing aspects they need to be holding
    CheckGossip {
        aol_idx: u64,
        response: tokio::sync::oneshot::Sender<CheckGossipData>,
    },
}

/// we asked for gossip - this indicates which aspects for which agents
/// should be requested
#[derive(Debug)]
pub struct CheckGossipData {
    pub spaces: im::HashMap<MonoSpaceHash, im::HashSet<MonoAgentId>>,
}

impl CheckGossipData {
    pub fn new() -> Self {
        Self {
            spaces: im::HashMap::new(),
        }
    }

    pub fn add_agent(&mut self, space_hash: MonoSpaceHash, agent_id: MonoAgentId) {
        let s = self.spaces.entry(space_hash).or_default();
        s.insert(agent_id);
    }

    pub fn spaces(self) -> im::HashMap<MonoSpaceHash, im::HashSet<MonoAgentId>> {
        self.spaces
    }
}

#[derive(Clone)]
struct UpcomingInstant(pub std::time::Instant);

impl std::fmt::Debug for UpcomingInstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = std::time::Instant::now();
        let tmp = match self.0.checked_duration_since(now) {
            Some(d) => format!("{:?} ms", d.as_millis()),
            None => "[expired]".to_string(),
        };
        f.debug_struct("UpcomingInstant").field("in", &tmp).finish()
    }
}

impl Serialize for UpcomingInstant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

impl UpcomingInstant {
    pub fn new_ms_from_now(ms: u64) -> Self {
        Self(
            std::time::Instant::now()
                .checked_add(std::time::Duration::from_millis(ms))
                .expect("instant"),
        )
    }

    pub fn still_pending(&self) -> bool {
        let now = std::time::Instant::now();
        self.0 > now
    }
}

/// protocol for sending messages to the `Store`
#[derive(Debug)]
enum StoreProto {
    Mutate(AolEntry, tokio::sync::oneshot::Sender<()>),
}

/// represents an active connection
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionState {
    agent_id: MonoAgentId,
    #[serde(serialize_with = "serialize_location")]
    agent_loc: Location,
    uri: MonoUri,
    next_gossip_check: UpcomingInstant,
}

pub type MonoAgentId = MonoRef<AgentId>;
pub type MonoSpaceHash = MonoRef<SpaceHash>;
pub type MonoEntryHash = MonoRef<EntryHash>;
pub type MonoAspectHash = MonoRef<AspectHash>;
pub type MonoUri = MonoRef<Lib3hUri>;

/// so we know who's holding what
pub type Holding = im::HashMap<MonoAspectHash, im::HashSet<MonoAgentId>>;

/// so we cache entry locations as well
#[derive(Debug, Clone, Serialize)]
pub struct EntryInfo {
    pub entry_hash: MonoEntryHash,
    #[serde(serialize_with = "serialize_location")]
    pub entry_loc: Location,
    pub aspects: Holding,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_location<S>(loc: &Location, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32(loc.clone().into())
}

/// sim2h state storage
#[derive(Serialize)]
pub struct Space {
    #[serde(skip)]
    pub crypto: Box<dyn CryptoSystem>,
    pub redundancy: u64,
    pub all_aspects: im::HashMap<MonoAspectHash, MonoAspectHash>,
    pub entry_to_all_aspects: im::HashMap<MonoEntryHash, EntryInfo>,
    pub connections: im::HashMap<MonoAgentId, ConnectionState>,
    pub uri_to_connection: im::HashMap<MonoUri, MonoAgentId>,
    pub gossip_interval: u64,
}

impl std::fmt::Debug for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("all_aspects", &self.all_aspects)
            .field("entry_to_all_aspects", &self.entry_to_all_aspects)
            .field("connections", &self.connections)
            .field("uri_to_connection", &self.uri_to_connection)
            .finish()
    }
}

impl Clone for Space {
    fn clone(&self) -> Self {
        Self {
            crypto: self.crypto.box_clone(),
            redundancy: self.redundancy,
            all_aspects: self.all_aspects.clone(),
            entry_to_all_aspects: self.entry_to_all_aspects.clone(),
            connections: self.connections.clone(),
            uri_to_connection: self.uri_to_connection.clone(),
            gossip_interval: self.gossip_interval,
        }
    }
}

impl Space {
    fn new(crypto: Box<dyn CryptoSystem>, redundancy: u64, gossip_interval: u64) -> Space {
        Space {
            crypto,
            redundancy,
            all_aspects: im::HashMap::new(),
            entry_to_all_aspects: im::HashMap::new(),
            connections: im::HashMap::new(),
            uri_to_connection: im::HashMap::new(),
            gossip_interval,
        }
    }

    fn get_mono_agent_id(&self, agent_id: &AgentId) -> MonoAgentId {
        match self.connections.get(agent_id) {
            None => agent_id.clone().into(),
            Some(c) => c.agent_id.clone(),
        }
    }

    fn get_entry_location(&self, entry_hash: &EntryHash) -> Location {
        if let Some(entry) = self.entry_to_all_aspects.get(entry_hash) {
            return entry.entry_loc;
        }
        entry_location(&self.crypto, entry_hash)
    }

    fn get_agents_that_should_hold_entry(
        &self,
        entry_hash: &EntryHash,
    ) -> im::HashSet<MonoAgentId> {
        if self.redundancy == 0 {
            // FULL SYNC
            return self.connections.keys().cloned().collect();
        }

        let entry_loc = self.get_entry_location(entry_hash);

        let mut out = im::HashSet::new();

        let agent_count = self.connections.len() as u64;

        for (agent_id, con) in self.connections.iter() {
            if should_store(con.agent_loc, entry_loc, agent_count, self.redundancy) {
                out.insert(agent_id.clone());
            }
        }

        out
    }

    fn get_agents_holding_entry(&self, entry_hash: &EntryHash) -> im::HashSet<MonoAgentId> {
        if let Some(entry) = self.entry_to_all_aspects.get(entry_hash) {
            let mut aspect_iter = entry.aspects.iter();
            let mut remaining_agents = match aspect_iter.next() {
                None => return im::HashSet::new(),
                Some((_, holding)) => holding.clone(),
            };

            for (_, holding) in aspect_iter {
                remaining_agents = remaining_agents.intersection(holding.clone());

                if remaining_agents.is_empty() {
                    return im::HashSet::new();
                }
            }

            return remaining_agents;
        }

        im::HashSet::new()
    }

    fn get_agents_that_need_aspect(
        &self,
        entry_hash: &EntryHash,
        aspect_hash: &AspectHash,
    ) -> im::HashSet<MonoAgentId> {
        let mut out = im::HashSet::new();
        if let Some(entry) = self.entry_to_all_aspects.get(entry_hash) {
            if let Some(holding) = entry.aspects.get(aspect_hash) {
                let agents = self.get_agents_that_should_hold_entry(entry_hash);
                for agent_id in agents {
                    if !holding.contains(&agent_id) {
                        out.insert(agent_id);
                    }
                }
            }
        }
        out
    }

    fn get_gossip_aspects_needed_for_agent(
        &self,
        agent_id: &AgentId,
    ) -> im::HashMap<MonoEntryHash, im::HashSet<MonoAspectHash>> {
        let mut out = im::HashMap::new();

        let agent_count = self.connections.len() as u64;
        let agent_loc = match self.connections.get(agent_id) {
            None => return out,
            Some(c) => c.agent_loc,
        };

        for (_, entry) in self.entry_to_all_aspects.iter() {
            if should_store(agent_loc, entry.entry_loc, agent_count, self.redundancy) {
                let e = out.entry(entry.entry_hash.clone()).or_default();
                for (aspect_hash, holding) in entry.aspects.iter() {
                    if !holding.contains(agent_id) {
                        e.insert(aspect_hash.clone());
                    }
                }
            }
        }

        out
    }

    fn check_insert_connection(&mut self, agent_id: &AgentId, uri: Lib3hUri) {
        let agent_id = self.get_mono_agent_id(agent_id);
        let uri: MonoUri = uri.into();

        let agent_loc =
            match lib3h::rrdht_util::calc_location_for_id(&self.crypto, &agent_id.to_string()) {
                Ok(loc) => loc,
                Err(e) => {
                    error!("FAILED to generate agent loc: {:?}", e);
                    return;
                }
            };

        // make it so we're almost ready to check this agent for gossip
        // (add 10 ms to make sure we don't do it right away...
        let to_add: u64 =
            thread_rng().gen_range(10, (self.gossip_interval as f64 * 0.25) as u64 + 10);

        let next_gossip_check = UpcomingInstant::new_ms_from_now(to_add);

        // - add the main connection entry
        match self.connections.entry(agent_id.clone()) {
            im::hashmap::Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                entry.agent_id = agent_id.clone();
                entry.agent_loc = agent_loc;
                entry.uri = uri.clone();
                entry.next_gossip_check = next_gossip_check;
            }
            im::hashmap::Entry::Vacant(entry) => {
                entry.insert(ConnectionState {
                    agent_id: agent_id.clone(),
                    agent_loc,
                    uri: uri.clone(),
                    next_gossip_check,
                });
            }
        }

        // - add entry to `uri_to_connection`
        self.uri_to_connection.insert(uri, agent_id.clone());

        // - clear all `holding` aspects
        self.clear_holding(&agent_id);
    }

    fn priv_check_insert_entry_hash(&mut self, entry_hash: &EntryHash) -> MonoEntryHash {
        if let Some(entry) = self.entry_to_all_aspects.get(entry_hash) {
            return entry.entry_hash.clone();
        }
        let entry_hash: MonoEntryHash = entry_hash.clone().into();
        let entry_loc = entry_location(&self.crypto, &entry_hash);
        let entry = EntryInfo {
            entry_hash: entry_hash.clone(),
            entry_loc,
            aspects: im::HashMap::new(),
        };
        self.entry_to_all_aspects.insert(entry_hash.clone(), entry);
        entry_hash
    }

    fn priv_check_insert_aspect(&mut self, aspect_hash: &AspectHash) -> MonoAspectHash {
        if let Some(a) = self.all_aspects.get(aspect_hash) {
            return a.clone();
        }
        let aspect_hash: MonoAspectHash = aspect_hash.clone().into();
        self.all_aspects
            .insert(aspect_hash.clone(), aspect_hash.clone());
        aspect_hash
    }

    fn agent_holds_aspects(
        &mut self,
        agent_id: &AgentId,
        entry_hash: &EntryHash,
        aspects: &im::HashSet<AspectHash>,
    ) {
        let agent_id = self.get_mono_agent_id(agent_id);
        let entry_hash = self.priv_check_insert_entry_hash(entry_hash);
        let mut mono_aspects = Vec::new();
        for aspect_hash in aspects {
            mono_aspects.push(self.priv_check_insert_aspect(aspect_hash));
        }
        let e = self.entry_to_all_aspects.get_mut(&entry_hash).unwrap();
        for a in mono_aspects {
            let holding = e.aspects.entry(a).or_default();
            holding.insert(agent_id.clone());
        }
    }

    fn clear_holding(&mut self, agent_id: &AgentId) {
        for entry in self.entry_to_all_aspects.iter_mut() {
            for holding_set in entry.aspects.iter_mut() {
                holding_set.remove(agent_id);
            }
        }
    }

    fn check_gossip(&mut self, space_hash: MonoSpaceHash, check_gossip_data: &mut CheckGossipData) {
        for con in self.connections.iter_mut() {
            if con.next_gossip_check.still_pending() {
                continue;
            }

            let to_add: u64 = thread_rng().gen_range(
                (self.gossip_interval as f64 * 0.75) as u64,
                (self.gossip_interval as f64 * 1.25) as u64,
            );

            con.next_gossip_check = UpcomingInstant::new_ms_from_now(to_add);

            check_gossip_data.add_agent(space_hash.clone(), con.agent_id.clone());
        }
    }
}

pub struct Store {
    pub crypto: Box<dyn CryptoSystem>,
    pub redundancy: u64,
    pub spaces: im::HashMap<MonoSpaceHash, Space>,
    pub con_incr: Arc<AtomicU64>,
    pub gossip_interval: u64,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("spaces", &self.spaces)
            .finish()
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Self {
            crypto: self.crypto.box_clone(),
            redundancy: self.redundancy,
            spaces: self.spaces.clone(),
            con_incr: self.con_incr.clone(),
            gossip_interval: self.gossip_interval,
        }
    }
}

impl Store {
    pub fn new(
        crypto: Box<dyn CryptoSystem>,
        redundancy: u64,
        gossip_interval: Option<u64>,
    ) -> StoreHandle {
        let (send_mut, mut recv_mut) = tokio::sync::mpsc::unbounded_channel();

        let ref_dummy = Arc::new(());

        let con_incr = Arc::new(AtomicU64::new(1));

        let weak_ref_dummy = Arc::downgrade(&ref_dummy);

        let mut store = Store {
            crypto,
            redundancy,
            spaces: im::HashMap::new(),
            con_incr: con_incr.clone(),
            gossip_interval: gossip_interval.unwrap_or(AGENT_FETCH_ASPECTS_INTERVAL_MS),
        };

        let clone_ref = Arc::new(tokio::sync::RwLock::new(store.clone()));

        let clone_ref_clone = clone_ref.clone();
        tokio::task::spawn(async move {
            let mut handle_message = move |msg| match msg {
                StoreProto::Mutate(aol_entry, complete) => {
                    store.mutate(aol_entry);
                    let store_clone = store.clone();
                    let clone_ref_clone_clone = clone_ref_clone.clone();
                    tokio::task::spawn(async move {
                        *clone_ref_clone_clone.write().await = store_clone;
                        let _ = complete.send(());
                    });
                }
            };
            'store_recv_loop: loop {
                if let None = weak_ref_dummy.upgrade() {
                    // there are no more references to us...
                    // let this task end
                    return;
                }

                match recv_mut.next().await {
                    // broken channel, let this task end
                    None => return,
                    Some(msg) => {
                        let loop_start = std::time::Instant::now();

                        handle_message(msg);

                        let mut count = 1;

                        // we've got some cpu time, process a batch of
                        // messages all at once if any more are pending
                        for _ in 0..100 {
                            match recv_mut.try_recv() {
                                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                                Err(tokio::sync::mpsc::error::TryRecvError::Closed) => {
                                    break 'store_recv_loop;
                                }
                                Ok(msg) => handle_message(msg),
                            }
                            count += 1;
                        }

                        trace!(
                            "sim2h_im_state main loop processed {} messages in {} ms",
                            count,
                            loop_start.elapsed().as_millis()
                        );
                    }
                }
            }
            warn!("im state store_recv_loop ended!");
        });

        StoreHandle::new(ref_dummy, clone_ref, send_mut, con_incr)
    }

    fn mutate(&mut self, aol_entry: AolEntry) {
        match aol_entry {
            AolEntry::NewConnection {
                aol_idx: _,
                space_hash,
                agent_id,
                uri,
            } => self.new_connection(space_hash, agent_id, uri),
            AolEntry::DropConnection {
                aol_idx: _,
                space_hash,
                agent_id,
            } => self.drop_connection(space_hash, agent_id),
            AolEntry::DropConnectionByUri { aol_idx: _, uri } => self.drop_connection_by_uri(uri),
            AolEntry::AgentHoldsAspects {
                aol_idx: _,
                space_hash,
                agent_id,
                entry_hash,
                aspects,
            } => self.agent_holds_aspects(space_hash, agent_id, entry_hash, aspects),
            AolEntry::CheckGossip {
                aol_idx: _,
                response,
            } => self.check_gossip(response),
        }
    }

    fn get_space(&self, space_hash: &SpaceHash) -> Option<&Space> {
        let space_hash: MonoSpaceHash = space_hash.clone().into();
        self.spaces.get(&space_hash)
    }

    fn get_space_mut(&mut self, space_hash: SpaceHash) -> &mut Space {
        if !self.spaces.contains_key(&space_hash) {
            let crypto = self.crypto.box_clone();
            self.spaces.insert(
                space_hash.clone().into(),
                Space::new(crypto, self.redundancy, self.gossip_interval),
            );
        }

        self.spaces.get_mut(&space_hash).unwrap()
    }

    fn new_connection(&mut self, space_hash: SpaceHash, agent_id: AgentId, uri: Lib3hUri) {
        self.get_space_mut(space_hash)
            .check_insert_connection(&agent_id, uri);
    }

    fn drop_connection_inner(space: &mut Space, agent_id: MonoAgentId) {
        // - clear all `holding` aspects (to prepare for another connection)
        space.clear_holding(&agent_id);

        // - remove main connection entry
        let uri = match space.connections.entry(agent_id) {
            im::hashmap::Entry::Occupied(entry) => entry.remove().uri,
            _ => return,
        };

        // - remove the uri_to_connection entry
        space.uri_to_connection.remove(&uri);
    }

    /// if there are no connections in a space, drop the space
    fn check_drop_spaces(&mut self) {
        let mut drop_spaces = Vec::new();

        for (space_hash, space) in self.spaces.iter() {
            if space.connections.is_empty() {
                drop_spaces.push(space_hash.clone());
            }
        }

        for space_hash in drop_spaces.drain(..) {
            self.spaces.remove(&space_hash);
        }
    }

    fn drop_connection(&mut self, space_hash: SpaceHash, agent_id: AgentId) {
        let agent_id: MonoAgentId = agent_id.into();

        let space = self.get_space_mut(space_hash);
        Self::drop_connection_inner(space, agent_id);
        self.check_drop_spaces();
    }

    fn drop_connection_by_uri(&mut self, uri: Lib3hUri) {
        for space in self.spaces.iter_mut() {
            let agent_id = match space.uri_to_connection.get(&uri) {
                Some(agent_id) => agent_id.clone(),
                None => continue,
            };

            Self::drop_connection_inner(space, agent_id);
        }
        self.check_drop_spaces();
    }

    fn agent_holds_aspects(
        &mut self,
        space_hash: SpaceHash,
        agent_id: AgentId,
        entry_hash: EntryHash,
        aspects: im::HashSet<AspectHash>,
    ) {
        self.get_space_mut(space_hash)
            .agent_holds_aspects(&agent_id, &entry_hash, &aspects);
    }

    fn check_gossip(&mut self, response: tokio::sync::oneshot::Sender<CheckGossipData>) {
        let mut check_gossip_data = CheckGossipData::new();

        let space_hashes = self.spaces.keys().cloned().collect::<Vec<_>>();
        for space_hash in space_hashes {
            self.spaces
                .get_mut(&space_hash)
                .unwrap()
                .check_gossip(space_hash, &mut check_gossip_data);
        }

        if let Err(e) = response.send(check_gossip_data) {
            error!("Failed to send check gossip response! {:?}", e);
        }
    }

    pub fn get_agents_that_need_aspect(
        &self,
        space_hash: &SpaceHash,
        entry_hash: &EntryHash,
        aspect_hash: &AspectHash,
    ) -> im::HashSet<MonoAgentId> {
        match self.get_space(space_hash) {
            None => im::HashSet::new(),
            Some(space) => space.get_agents_that_need_aspect(entry_hash, aspect_hash),
        }
    }

    pub fn get_gossip_aspects_needed_for_agent(
        &self,
        space_hash: &SpaceHash,
        agent_id: &AgentId,
    ) -> Option<im::HashMap<MonoEntryHash, im::HashSet<MonoAspectHash>>> {
        let space = self.get_space(space_hash)?;
        Some(space.get_gossip_aspects_needed_for_agent(agent_id))
    }

    /// how many spaces do we currently have registered?
    pub fn spaces_count(&self) -> usize {
        self.spaces.len()
    }

    /// if we have an active connection for an agent_id - get the uri
    pub fn lookup_joined(&self, space_hash: &SpaceHash, agent_id: &AgentId) -> Option<&Lib3hUri> {
        let agent_id: MonoAgentId = agent_id.clone().into();
        let space = self.get_space(space_hash)?;
        let con = space.connections.get(&agent_id)?;
        Some(&con.uri)
    }

    /// sim2h is currently NOT set up to handle multiple spaces per connection
    /// while this should be fixed, for now we need to support the current
    /// use-case. Just returning the first con/agent encountered in spaces.
    pub fn get_space_info_from_uri(&self, uri: &Lib3hUri) -> Option<(MonoAgentId, MonoSpaceHash)> {
        for (space_hash, space) in self.spaces.iter() {
            match space.uri_to_connection.get(uri) {
                None => continue,
                Some(agent_id) => {
                    return Some((agent_id.clone(), space_hash.clone()));
                }
            }
        }
        None
    }

    pub fn get_agents_that_should_hold_entry(
        &self,
        space_hash: &SpaceHash,
        entry_hash: &EntryHash,
    ) -> Option<im::HashSet<MonoAgentId>> {
        let space = self.get_space(space_hash)?;
        Some(space.get_agents_that_should_hold_entry(entry_hash))
    }

    pub fn get_agents_holding_entry(
        &self,
        space_hash: &SpaceHash,
        entry_hash: &EntryHash,
    ) -> Option<im::HashSet<MonoAgentId>> {
        let space = self.get_space(space_hash)?;
        Some(space.get_agents_holding_entry(entry_hash))
    }

    pub fn get_agents_for_query(
        &self,
        space_hash: &SpaceHash,
        entry_hash: &EntryHash,
        requesting_agent_id: Option<&AgentId>,
    ) -> Vec<MonoAgentId> {
        // first check agents we believe are actually holding this data
        let mut out = match self.get_agents_holding_entry(space_hash, entry_hash) {
            None => im::HashSet::new(),
            Some(s) => s,
        };
        if let Some(aid) = requesting_agent_id {
            out.remove(aid);
        }
        if out.is_empty() {
            // if we don't already have good options, get the list of
            // those who MAY be holding the data
            if let Some(s) = self.get_agents_that_should_hold_entry(space_hash, entry_hash) {
                out = s;
            }
            if let Some(aid) = requesting_agent_id {
                out.remove(aid);
            }
        }
        if out.is_empty() {
            if requesting_agent_id.is_none() {
                // there are no options
                return vec![];
            }
            // only in the case where we have no other options
            // do we want to redirect the query back to the requester
            out.insert(MonoAgentId::new(requesting_agent_id.unwrap().clone()));
        }
        let mut out: Vec<MonoAgentId> = out.iter().cloned().collect();
        let out = &mut out[..];
        out.shuffle(&mut thread_rng());
        out[..std::cmp::min(out.len(), 3)].to_vec()
    }
}

/// wrapper around a Store clone making it look read only,
/// without this one might be tempted to mutate something in a temporary clone
#[derive(Clone)]
pub struct StoreRef(Store);

impl std::fmt::Debug for StoreRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl std::ops::Deref for StoreRef {
    type Target = Store;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::convert::AsRef<Store> for StoreRef {
    fn as_ref(&self) -> &Store {
        &self.0
    }
}

impl std::borrow::Borrow<Store> for StoreRef {
    fn borrow(&self) -> &Store {
        &self.0
    }
}

#[derive(Clone)]
/// give us a cheaply clone-able async handle to the real store
pub struct StoreHandle {
    // this is just used for ref-counting
    _ref_dummy: Arc<()>,
    // clone ref
    clone_ref: Arc<tokio::sync::RwLock<Store>>,
    send_mut: tokio::sync::mpsc::UnboundedSender<StoreProto>,
    con_incr: Arc<AtomicU64>,
}

impl StoreHandle {
    fn new(
        ref_dummy: Arc<()>,
        clone_ref: Arc<tokio::sync::RwLock<Store>>,
        send_mut: tokio::sync::mpsc::UnboundedSender<StoreProto>,
        con_incr: Arc<AtomicU64>,
    ) -> Self {
        Self {
            _ref_dummy: ref_dummy,
            clone_ref,
            send_mut,
            con_incr,
        }
    }

    pub async fn get_clone(&self) -> StoreRef {
        StoreRef(self.clone_ref.read().await.clone())
    }

    #[must_use]
    pub fn new_connection(
        &self,
        space_hash: SpaceHash,
        agent_id: AgentId,
        uri: Lib3hUri,
    ) -> BoxFuture<'static, ()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let msg = StoreProto::Mutate(
            AolEntry::NewConnection {
                aol_idx: self.con_incr.inc(),
                space_hash,
                agent_id,
                uri,
            },
            sender,
        );
        if let Err(_) = self.send_mut.send(msg) {
            error!("failed to send im store message - shutting down?");
            return async { () }.boxed();
        }
        async move {
            let _ = receiver.await;
        }
        .boxed()
    }

    pub fn spawn_new_connection(&self, space_hash: SpaceHash, agent_id: AgentId, uri: Lib3hUri) {
        let f = self.new_connection(space_hash, agent_id, uri);
        tokio::task::spawn(f);
    }

    /*
    #[must_use]
    pub fn drop_connection(&self, space_hash: SpaceHash, agent_id: AgentId) -> BoxFuture<'static, ()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        if let Err(_) = self
            .send_mut
            .send(StoreProto::Mutate(AolEntry::DropConnection {
                aol_idx: self.con_incr.inc(),
                space_hash,
                agent_id,
            }, sender))
        {
            error!("failed to send im store message - shutting down?");
            return async { () }.boxed();
        }
        async move {
            let _ = receiver.await;
        }.boxed()
    }

    pub fn spawn_drop_connection(&self, space_hash: SpaceHash, agent_id: AgentId) {
        let f = self.drop_connection(space_hash, agent_id);
        tokio::task::spawn(f);
    }
    */

    #[must_use]
    pub fn drop_connection_by_uri(&self, uri: Lib3hUri) -> BoxFuture<'static, ()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        if let Err(_) = self.send_mut.send(StoreProto::Mutate(
            AolEntry::DropConnectionByUri {
                aol_idx: self.con_incr.inc(),
                uri,
            },
            sender,
        )) {
            error!("failed to send im store message - shutting down?");
            return async { () }.boxed();
        }
        async move {
            let _ = receiver.await;
        }
        .boxed()
    }

    pub fn spawn_drop_connection_by_uri(&self, uri: Lib3hUri) {
        let f = self.drop_connection_by_uri(uri);
        tokio::task::spawn(f);
    }

    #[must_use]
    pub fn agent_holds_aspects(
        &self,
        space_hash: SpaceHash,
        agent_id: AgentId,
        entry_hash: EntryHash,
        aspects: im::HashSet<AspectHash>,
    ) -> BoxFuture<'static, ()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        if let Err(_) = self.send_mut.send(StoreProto::Mutate(
            AolEntry::AgentHoldsAspects {
                aol_idx: self.con_incr.inc(),
                space_hash,
                agent_id,
                entry_hash,
                aspects,
            },
            sender,
        )) {
            error!("failed to send im store message - shutting down?");
            return async { () }.boxed();
        }
        async move {
            let _ = receiver.await;
        }
        .boxed()
    }

    pub fn spawn_agent_holds_aspects(
        &self,
        space_hash: SpaceHash,
        agent_id: AgentId,
        entry_hash: EntryHash,
        aspects: im::HashSet<AspectHash>,
    ) {
        let f = self.agent_holds_aspects(space_hash, agent_id, entry_hash, aspects);
        tokio::task::spawn(f);
    }

    pub async fn check_gossip(&self) -> CheckGossipData {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let (sender_c, receiver_c) = tokio::sync::oneshot::channel();
        if let Err(_) = self.send_mut.send(StoreProto::Mutate(
            AolEntry::CheckGossip {
                aol_idx: self.con_incr.inc(),
                response: sender,
            },
            sender_c,
        )) {
            error!("failed to send im store message - shutting down?");
            // we're probably shutting down, prevent panic!s
            // note this future will never resolve - because it cannot
            return futures::future::pending().await;
        }
        let _ = receiver_c.await;
        receiver.await.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn async_run(f: BoxFuture<'static, ()>) {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "trace");
        }
        let _ = env_logger::builder()
            .default_format_timestamp(false)
            .default_format_module_path(false)
            .is_test(true)
            .try_init();
        tokio::runtime::Builder::new()
            .threaded_scheduler()
            .core_threads(num_cpus::get())
            .thread_name("tokio-rkv-test-thread")
            .enable_all()
            .build()
            .unwrap()
            .block_on(f);
    }

    fn gen_agent() -> AgentId {
        let crypto = Box::new(lib3h_sodium::SodiumCryptoSystem::new());
        let enc = hcid::HcidEncoding::with_kind("hcs0").unwrap();

        let mut pk1 = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
        let mut sk1 = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
        crypto.sign_keypair(&mut pk1, &mut sk1).unwrap();

        enc.encode(&*pk1).unwrap().into()
    }

    async fn async_workflow_test() {
        let aid1 = gen_agent();
        let aid2 = gen_agent();

        let space_hash: SpaceHash = "abcd".into();
        let uri1: Lib3hUri = url::Url::parse("ws://yada1").unwrap().into();
        let uri2: Lib3hUri = url::Url::parse("ws://yada2").unwrap().into();

        let crypto = Box::new(lib3h_sodium::SodiumCryptoSystem::new());
        let store = Store::new(
            crypto, 0,    /* full sync */
            None, /* default gossip */
        );

        debug!("GOT: {:#?}", store.get_clone().await);

        assert_eq!(
            None,
            store
                .get_clone()
                .await
                .lookup_joined(&space_hash, &"id-1".into(),)
        );
        store
            .new_connection(space_hash.clone(), aid1.clone(), uri1.clone())
            .await;
        assert_eq!(
            Some(&uri1),
            store.get_clone().await.lookup_joined(&space_hash, &aid1,)
        );
        store
            .new_connection(space_hash.clone(), aid2.clone(), uri2.clone())
            .await;

        debug!("GOT: {:#?}", store.get_clone().await);

        store
            .agent_holds_aspects(
                space_hash.clone(),
                aid1.clone(),
                "test".into(),
                im::hashset! {"one".into(), "two".into()},
            )
            .await;
        store
            .agent_holds_aspects(
                space_hash.clone(),
                aid2.clone(),
                "test".into(),
                im::hashset! {"one".into()},
            )
            .await;

        debug!("GOT: {:#?}", store.get_clone().await);

        debug!("--- beg check missing ---");
        let store_clone = store.get_clone().await;
        for (space_hash, space) in store_clone.spaces.iter() {
            assert_eq!("SpaceHash(abcd)", &format!("{:?}", space_hash),);
            debug!("-- space: {:?} --", space_hash);
            for (agent_id, _c) in space.connections.iter() {
                assert!(**agent_id == aid1 || **agent_id == aid2);
                debug!("-- agent: {:?} --", agent_id);
                let res = store_clone.get_gossip_aspects_needed_for_agent(&space_hash, &agent_id);
                debug!("{:#?}", res,);
                if **agent_id == aid1 {
                    assert_eq!("Some({EntryHash(test): {}})", &format!("{:?}", res),);
                } else if **agent_id == aid2 {
                    assert_eq!(
                        "Some({EntryHash(test): {AspectHash(two)}})",
                        &format!("{:?}", res),
                    );
                }
            }
        }
        debug!("--- end check missing ---");

        //store.drop_connection(space_hash.clone(), aid1.clone());
        store.drop_connection_by_uri(uri1.clone()).await;

        debug!("GOT: {:#?}", store.get_clone().await);
    }

    #[test]
    fn workflow_test() {
        async_run(async_workflow_test().boxed());
    }

    async fn async_same_aspect_in_differing_entries_test() {
        let aid1 = gen_agent();

        let space_hash: SpaceHash = "abcd".into();
        let entry_hash_1: EntryHash = "test1".into();
        let entry_hash_2: EntryHash = "test2".into();
        let aspect_hash: AspectHash = "one".into();
        let uri1: Lib3hUri = url::Url::parse("ws://yada1").unwrap().into();

        let crypto = Box::new(lib3h_sodium::SodiumCryptoSystem::new());
        let store = Store::new(
            crypto,
            0,       // FULL SYNC
            Some(6), // set a nice/short 6ms gossip interval for testing : )
        );

        store
            .new_connection(space_hash.clone(), aid1.clone(), uri1.clone())
            .await;

        store
            .agent_holds_aspects(
                space_hash.clone(),
                aid1.clone(),
                entry_hash_1.clone(),
                im::hashset! {aspect_hash.clone()},
            )
            .await;

        store
            .agent_holds_aspects(
                space_hash.clone(),
                aid1.clone(),
                entry_hash_2.clone(),
                im::hashset! {aspect_hash.clone()},
            )
            .await;

        let state = store.get_clone().await;
        debug!("GOT: {:#?}", state);

        let space = state.spaces.get(&space_hash).unwrap();

        assert_eq!(1, space.all_aspects.len());
        assert_eq!(2, space.entry_to_all_aspects.len());
    }

    #[test]
    fn same_aspect_in_differing_entries_test() {
        async_run(async_same_aspect_in_differing_entries_test().boxed());
    }

    async fn async_gossip_test() {
        let aid1 = gen_agent();
        let aid2 = gen_agent();

        let space_hash: SpaceHash = "abcd".into();
        let entry_hash: EntryHash = "test".into();
        let aspect_hash_1: AspectHash = "one".into();
        let aspect_hash_2: AspectHash = "two".into();
        let uri1: Lib3hUri = url::Url::parse("ws://yada1").unwrap().into();
        let uri2: Lib3hUri = url::Url::parse("ws://yada2").unwrap().into();

        let crypto = Box::new(lib3h_sodium::SodiumCryptoSystem::new());
        let store = Store::new(
            crypto,
            0,       // FULL SYNC
            Some(6), // set a nice/short 6ms gossip interval for testing : )
        );

        store
            .new_connection(space_hash.clone(), aid1.clone(), uri1.clone())
            .await;
        store
            .new_connection(space_hash.clone(), aid2.clone(), uri2.clone())
            .await;

        debug!("GOT: {:#?}", store.get_clone().await);

        store
            .agent_holds_aspects(
                space_hash.clone(),
                aid1.clone(),
                entry_hash.clone(),
                im::hashset! {aspect_hash_1.clone(), aspect_hash_2.clone()},
            )
            .await;

        debug!("GOT: {:#?}", store.get_clone().await);

        // give us time to need gossip
        tokio::time::delay_for(std::time::Duration::from_millis(17)).await;

        let need_gossip = store.check_gossip().await;

        debug!("GOT: {:#?}", need_gossip);

        assert!(need_gossip.spaces.get(&space_hash).unwrap().contains(&aid1));
        assert!(need_gossip.spaces.get(&space_hash).unwrap().contains(&aid2));

        let store_clone = store.get_clone().await;

        let res = store_clone.get_gossip_aspects_needed_for_agent(&space_hash, &aid1);

        debug!("GOT: {:#?}", res);

        assert_eq!(0, res.unwrap().get(&entry_hash).unwrap().len());

        let res = store_clone.get_gossip_aspects_needed_for_agent(&space_hash, &aid2);

        debug!("GOT: {:#?}", res);

        assert_eq!(2, res.as_ref().unwrap().get(&entry_hash).unwrap().len());
        assert!(res
            .as_ref()
            .unwrap()
            .get(&entry_hash)
            .unwrap()
            .contains(&aspect_hash_1));
        assert!(res
            .as_ref()
            .unwrap()
            .get(&entry_hash)
            .unwrap()
            .contains(&aspect_hash_2));
    }

    #[test]
    fn gossip_test() {
        async_run(async_gossip_test().boxed());
    }
}
