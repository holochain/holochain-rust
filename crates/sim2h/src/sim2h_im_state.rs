use crate::*;
use lib3h::rrdht_util::Location;
use rand::Rng;
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
/// note this value is randomized by ~50% each time
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
    DropConnection {
        aol_idx: u64,
        space_hash: SpaceHash,
        agent_id: AgentId,
    },

    // we need to be able to drop all connectios across spaces based on
    // the uri of the connected socket (i.e. in case of a socket read/write err)
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

/// protocol for sending messages to the `Store`
#[derive(Debug)]
enum StoreProto {
    GetClone(tokio::sync::oneshot::Sender<Store>),
    Mutate(AolEntry),
}

/// represents an active connection
#[derive(Debug, Clone)]
pub struct ConnectionState {
    agent_id: MonoAgentId,
    agent_loc: Location,
    uri: MonoUri,
    next_gossip_check: std::time::Instant,
}

pub type MonoAgentId = MonoRef<AgentId>;
pub type MonoSpaceHash = MonoRef<SpaceHash>;
pub type MonoEntryHash = MonoRef<EntryHash>;
pub type MonoAspectHash = MonoRef<AspectHash>;
pub type MonoUri = MonoRef<Lib3hUri>;

/// so we know who's holding what
pub type Holding = im::HashMap<MonoAspectHash, im::HashSet<MonoAgentId>>;

/// so we cache entry locations as well
#[derive(Debug, Clone)]
pub struct Entry {
    pub entry_hash: MonoEntryHash,
    pub entry_loc: Location,
    pub aspects: Holding,
}

/// sim2h state storage
pub struct Space {
    pub crypto: Box<dyn CryptoSystem>,
    pub redundancy: u64,
    pub aspect_to_entry_hash: im::HashMap<MonoAspectHash, (MonoAspectHash, MonoEntryHash)>,
    pub entry_to_all_aspects: im::HashMap<MonoEntryHash, Entry>,
    pub connections: im::HashMap<MonoAgentId, ConnectionState>,
    pub uri_to_connection: im::HashMap<MonoUri, MonoAgentId>,
}

impl std::fmt::Debug for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("aspect_to_entry_hash", &self.aspect_to_entry_hash)
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
            aspect_to_entry_hash: self.aspect_to_entry_hash.clone(),
            entry_to_all_aspects: self.entry_to_all_aspects.clone(),
            connections: self.connections.clone(),
            uri_to_connection: self.uri_to_connection.clone(),
        }
    }
}

impl Space {
    fn new(crypto: Box<dyn CryptoSystem>, redundancy: u64) -> Space {
        Space {
            crypto,
            redundancy,
            aspect_to_entry_hash: im::HashMap::new(),
            entry_to_all_aspects: im::HashMap::new(),
            connections: im::HashMap::new(),
            uri_to_connection: im::HashMap::new(),
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
        let to_add: u64 = thread_rng().gen_range(0, AGENT_FETCH_ASPECTS_INTERVAL_MS / 4);

        let next_gossip_check = std::time::Instant::now()
            .checked_add(std::time::Duration::from_millis(to_add))
            .unwrap();

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
        let entry = Entry {
            entry_hash: entry_hash.clone(),
            entry_loc,
            aspects: im::HashMap::new(),
        };
        self.entry_to_all_aspects.insert(entry_hash.clone(), entry);
        entry_hash
    }

    fn priv_check_insert_aspect_to_entry(
        &mut self,
        entry_hash: MonoEntryHash,
        aspect_hash: &AspectHash,
    ) -> MonoAspectHash {
        if let Some((a, e)) = self.aspect_to_entry_hash.get(aspect_hash) {
            if e != &entry_hash {
                panic!("entry/aspect mismatch - corrupted data?");
            }
            return a.clone();
        }
        let aspect_hash: MonoAspectHash = aspect_hash.clone().into();
        self.aspect_to_entry_hash
            .insert(aspect_hash.clone(), (aspect_hash.clone(), entry_hash));
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
            mono_aspects
                .push(self.priv_check_insert_aspect_to_entry(entry_hash.clone(), aspect_hash));
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
        let now = std::time::Instant::now();

        for con in self.connections.iter_mut() {
            if con.next_gossip_check > now {
                continue;
            }

            let to_add: u64 = thread_rng().gen_range(0, AGENT_FETCH_ASPECTS_INTERVAL_MS / 2)
                + ((AGENT_FETCH_ASPECTS_INTERVAL_MS * 4) / 3);

            con.next_gossip_check = now
                .checked_add(std::time::Duration::from_millis(to_add))
                .unwrap();

            check_gossip_data.add_agent(space_hash.clone(), con.agent_id.clone());
        }
    }
}

pub struct Store {
    pub crypto: Box<dyn CryptoSystem>,
    pub redundancy: u64,
    pub spaces: im::HashMap<MonoSpaceHash, Space>,
    pub con_incr: Arc<AtomicU64>,
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
        }
    }
}

impl Store {
    pub fn new(crypto: Box<dyn CryptoSystem>, redundancy: u64) -> StoreHandle {
        let (send_mut, mut recv_mut) = tokio::sync::mpsc::unbounded_channel();

        let ref_dummy = Arc::new(());

        let con_incr = Arc::new(AtomicU64::new(1));

        let weak_ref_dummy = Arc::downgrade(&ref_dummy);

        let mut store = Store {
            crypto,
            redundancy,
            spaces: im::HashMap::new(),
            con_incr: con_incr.clone(),
        };

        tokio::task::spawn(async move {
            let mut should_end_task = false;
            loop {
                if let None = weak_ref_dummy.upgrade() {
                    // there are no more references to us...
                    // let this task end
                    return;
                }

                match recv_mut.next().await {
                    // broken channel, let this task end
                    None => return,
                    Some(msg) => {
                        let mut messages = vec![msg];

                        // we've got some cpu time, process a batch of
                        // messages all at once if any more are pending
                        for _ in 0..100 {
                            match recv_mut.try_recv() {
                                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                                Err(tokio::sync::mpsc::error::TryRecvError::Closed) => {
                                    should_end_task = true;
                                    break;
                                }
                                Ok(msg) => messages.push(msg),
                            }
                        }

                        for msg in messages.drain(..) {
                            match msg {
                                StoreProto::GetClone(sender) => {
                                    sender.send(store.clone()).unwrap();
                                }
                                StoreProto::Mutate(aol_entry) => {
                                    store.mutate(aol_entry);
                                }
                            }
                        }

                        // if we got a Closed on our recv
                        if should_end_task {
                            return;
                        }
                    }
                }
            }
        });

        StoreHandle::new(ref_dummy, send_mut, con_incr)
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
                Space::new(crypto, self.redundancy),
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

    fn drop_connection(&mut self, space_hash: SpaceHash, agent_id: AgentId) {
        let agent_id: MonoAgentId = agent_id.into();

        let space = self.get_space_mut(space_hash);
        Self::drop_connection_inner(space, agent_id);
    }

    fn drop_connection_by_uri(&mut self, uri: Lib3hUri) {
        for space in self.spaces.iter_mut() {
            let agent_id = match space.uri_to_connection.get(&uri) {
                Some(agent_id) => agent_id.clone(),
                None => continue,
            };

            Self::drop_connection_inner(space, agent_id);
        }
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
    send_mut: tokio::sync::mpsc::UnboundedSender<StoreProto>,
    con_incr: Arc<AtomicU64>,
}

impl StoreHandle {
    fn new(
        ref_dummy: Arc<()>,
        send_mut: tokio::sync::mpsc::UnboundedSender<StoreProto>,
        con_incr: Arc<AtomicU64>,
    ) -> Self {
        Self {
            _ref_dummy: ref_dummy,
            send_mut,
            con_incr,
        }
    }

    pub async fn get_clone(&self) -> StoreRef {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        if let Err(_) = self.send_mut.send(StoreProto::GetClone(sender)) {
            // we're probably shutting down, prevent panic!s
            return futures::future::pending().await;
        }
        StoreRef(receiver.await.unwrap())
    }

    pub fn new_connection(&self, space_hash: SpaceHash, agent_id: AgentId, uri: Lib3hUri) {
        let msg = StoreProto::Mutate(AolEntry::NewConnection {
            aol_idx: self.con_incr.inc(),
            space_hash,
            agent_id,
            uri,
        });
        let _ = self.send_mut.send(msg);
    }

    pub fn drop_connection(&self, space_hash: SpaceHash, agent_id: AgentId) {
        let _ = self
            .send_mut
            .send(StoreProto::Mutate(AolEntry::DropConnection {
                aol_idx: self.con_incr.inc(),
                space_hash,
                agent_id,
            }));
    }

    pub fn drop_connection_by_uri(&self, uri: Lib3hUri) {
        let _ = self
            .send_mut
            .send(StoreProto::Mutate(AolEntry::DropConnectionByUri {
                aol_idx: self.con_incr.inc(),
                uri,
            }));
    }

    pub fn agent_holds_aspects(
        &self,
        space_hash: SpaceHash,
        agent_id: AgentId,
        entry_hash: EntryHash,
        aspects: im::HashSet<AspectHash>,
    ) {
        let _ = self
            .send_mut
            .send(StoreProto::Mutate(AolEntry::AgentHoldsAspects {
                aol_idx: self.con_incr.inc(),
                space_hash,
                agent_id,
                entry_hash,
                aspects,
            }));
    }

    pub async fn check_gossip(&self) -> CheckGossipData {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        if let Err(_) = self
            .send_mut
            .send(StoreProto::Mutate(AolEntry::CheckGossip {
                aol_idx: self.con_incr.inc(),
                response: sender,
            }))
        {
            // we're probably shutting down, prevent panic!s
            return futures::future::pending().await;
        }
        receiver.await.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn async_workflow_test() {
        let crypto = Box::new(lib3h_sodium::SodiumCryptoSystem::new());
        let enc = hcid::HcidEncoding::with_kind("hcs0").unwrap();

        let mut pk1 = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
        let mut sk1 = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
        crypto.sign_keypair(&mut pk1, &mut sk1).unwrap();

        let aid1: AgentId = enc.encode(&*pk1).unwrap().into();

        let mut pk2 = crypto.buf_new_insecure(crypto.sign_public_key_bytes());
        let mut sk2 = crypto.buf_new_secure(crypto.sign_secret_key_bytes());
        crypto.sign_keypair(&mut pk2, &mut sk2).unwrap();

        let aid2: AgentId = enc.encode(&*pk2).unwrap().into();

        let space_hash: SpaceHash = "abcd".into();
        let uri1: Lib3hUri = url::Url::parse("ws://yada1").unwrap().into();
        let uri2: Lib3hUri = url::Url::parse("ws://yada2").unwrap().into();

        let store = Store::new(crypto, 0);

        println!("GOT: {:#?}", store.get_clone().await);

        assert_eq!(
            None,
            store
                .get_clone()
                .await
                .lookup_joined(&space_hash, &"id-1".into(),)
        );
        store.new_connection(space_hash.clone(), aid1.clone(), uri1.clone());
        assert_eq!(
            Some(&uri1),
            store.get_clone().await.lookup_joined(&space_hash, &aid1,)
        );
        store.new_connection(space_hash.clone(), aid2.clone(), uri2.clone());

        println!("GOT: {:#?}", store.get_clone().await);

        store.agent_holds_aspects(
            space_hash.clone(),
            aid1.clone(),
            "test".into(),
            im::hashset! {"one".into(), "two".into()},
        );
        store.agent_holds_aspects(
            space_hash.clone(),
            aid2.clone(),
            "test".into(),
            im::hashset! {"one".into()},
        );

        println!("GOT: {:#?}", store.get_clone().await);

        println!("--- beg check missing ---");
        let store_clone = store.get_clone().await;
        for (space_hash, space) in store_clone.spaces.iter() {
            println!("-- space: {:?} --", space_hash);
            for (agent_id, _c) in space.connections.iter() {
                println!("-- agent: {:?} --", agent_id);
                println!(
                    "{:#?}",
                    store_clone.get_gossip_aspects_needed_for_agent(&space_hash, &agent_id),
                );
            }
        }
        println!("--- end check missing ---");

        store.drop_connection(space_hash.clone(), aid1.clone());

        println!("GOT: {:#?}", store.get_clone().await);
    }

    #[test]
    fn workflow_test() {
        tokio::runtime::Builder::new()
            .threaded_scheduler()
            .core_threads(num_cpus::get())
            .thread_name("tokio-rkv-test-thread")
            .enable_all()
            .build()
            .unwrap()
            .block_on(async_workflow_test());
    }
}
