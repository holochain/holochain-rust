use crate::*;
use lib3h::rrdht_util::Location;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::stream::StreamExt;

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
    agent_loc: Location,
    uri: Lib3hUri,
}

pub type MonoAgentId = MonoRef<String>;
pub type MonoSpaceHash = MonoRef<String>;
pub type MonoEntryHash = MonoRef<String>;
pub type MonoAspectHash = MonoRef<String>;

/// so we cache entry locations as well
#[derive(Debug, Clone)]
pub struct Entry {
    entry_loc: Location,
    aspects: im::HashSet<MonoAspectHash>,
}

/// sim2h state storage
#[derive(Debug, Clone)]
pub struct Space {
    pub all_aspects: im::HashMap<MonoEntryHash, Entry>,
    pub connections: im::HashMap<MonoAgentId, ConnectionState>,
    pub uri_to_connection: im::HashMap<Lib3hUri, MonoAgentId>,
    pub holding: im::HashMap<MonoAspectHash, im::HashSet<MonoAgentId>>,
}

impl Space {
    fn new() -> Space {
        Space {
            all_aspects: im::HashMap::new(),
            connections: im::HashMap::new(),
            uri_to_connection: im::HashMap::new(),
            holding: im::HashMap::new(),
        }
    }
}

pub struct Store {
    pub crypto: Box<dyn CryptoSystem>,
    pub spaces: im::HashMap<MonoSpaceHash, Space>,
    pub con_incr: Arc<AtomicU64>,
    mono_ref_cache: Option<MonoRefCache<String>>,
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
            spaces: self.spaces.clone(),
            con_incr: self.con_incr.clone(),
            mono_ref_cache: self.mono_ref_cache.clone(),
        }
    }
}

impl Store {
    pub fn new(crypto: Box<dyn CryptoSystem>) -> StoreHandle {
        let (send_mut, mut recv_mut) = tokio::sync::mpsc::unbounded_channel();

        let ref_dummy = Arc::new(());

        let con_incr = Arc::new(AtomicU64::new(1));

        let weak_ref_dummy = Arc::downgrade(&ref_dummy);

        let mut store = Store {
            crypto,
            spaces: im::HashMap::new(),
            con_incr: con_incr.clone(),
            mono_ref_cache: Some(MonoRefCache::new()),
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
        }
    }

    fn get_space(&self, space_hash: &SpaceHash) -> Option<&Space> {
        let space_hash = self.mono_get(space_hash.clone().into());
        self.spaces.get(&space_hash)
    }

    fn get_space_mut(&mut self, space_hash: SpaceHash) -> &mut Space {
        let space_hash = self.mono_get(space_hash.into());
        self.spaces
            .entry(space_hash)
            .or_insert_with(|| Space::new())
    }

    fn mono_get(&self, s: String) -> MonoRef<String> {
        self.mono_ref_cache.as_ref().unwrap().get(s)
    }

    fn ensure_aspects(
        &mut self,
        space_hash: &SpaceHash,
        entry_hash: &EntryHash,
        aspects: &im::HashSet<AspectHash>,
    ) {
        let entry_hash = self.mono_get(entry_hash.clone().into());
        let need_entry = {
            let space = self.get_space_mut(space_hash.clone());
            !space.all_aspects.contains_key(&entry_hash)
        };

        if need_entry {
            let entry_loc = entry_location(&self.crypto, &entry_hash.as_entry_hash());
            let space = self.get_space_mut(space_hash.clone());
            space.all_aspects.insert(
                entry_hash.clone(),
                Entry {
                    entry_loc,
                    aspects: im::HashSet::new(),
                },
            );
        }

        let space = self.get_space_mut(space_hash.clone());
        let e = space.all_aspects.get_mut(&entry_hash).unwrap();

        for a in aspects {
            e.aspects.insert(a.clone().into());
        }
    }

    fn new_connection(&mut self, space_hash: SpaceHash, agent_id: AgentId, uri: Lib3hUri) {
        let agent_loc =
            match lib3h::rrdht_util::calc_location_for_id(&self.crypto, &agent_id.to_string()) {
                Ok(loc) => loc,
                Err(e) => {
                    error!("FAILED to generate agent loc: {:?}", e);
                    return;
                }
            };
        let agent_id = self.mono_get(agent_id.into());

        let space = self.get_space_mut(space_hash);

        // - set connections entry to is_connected=true
        match space.connections.entry(agent_id.clone()) {
            im::hashmap::Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                entry.agent_loc = agent_loc;
                entry.uri = uri.clone();
            }
            im::hashmap::Entry::Vacant(entry) => {
                entry.insert(ConnectionState {
                    agent_loc,
                    uri: uri.clone(),
                });
            }
        }

        // - add entry to `uri_to_connection`
        space.uri_to_connection.insert(uri, agent_id);
        // - TODO clear `holding`?
    }

    fn drop_connection_inner(space: &mut Space, agent_id: MonoAgentId) {
        // - mark connection as disconnected (tombstone)
        let uri = match space.connections.entry(agent_id.clone()) {
            im::hashmap::Entry::Occupied(entry) => entry.remove().uri,
            _ => return,
        };
        // - remove the uri_to_connection entry
        space.uri_to_connection.remove(&uri);
        // - clear all `holding` aspects (to prepare for another connection
        for h in space.holding.iter_mut() {
            h.remove(&agent_id);
        }
    }

    fn drop_connection(&mut self, space_hash: SpaceHash, agent_id: AgentId) {
        let agent_id = self.mono_get(agent_id.into());

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
        self.ensure_aspects(&space_hash, &entry_hash, &aspects);
        let agent_id = self.mono_get(agent_id.into());

        let space = self.get_space_mut(space_hash);

        if !space.connections.contains_key(&agent_id) {
            return;
        }
        for aspect in aspects {
            space
                .holding
                .entry(aspect.into())
                .or_default()
                .insert(agent_id.clone());
        }
    }

    /// if we have an active connection for an agent_id - get the uri
    pub fn lookup_joined(&self, space_hash: &SpaceHash, agent_id: &AgentId) -> Option<&Lib3hUri> {
        let agent_id = self.mono_get(agent_id.into());
        let space = self.get_space(space_hash)?;
        let con = space.connections.get(&agent_id)?;
        Some(&con.uri)
    }

    /// return a mapping of all entry_hash/aspect_hashes
    /// that each agent is missing (note how it returns references :+1:)
    pub fn get_agents_missing_aspects(
        &self,
        space_hash: &SpaceHash,
    ) -> im::HashMap<MonoAgentId, im::HashMap<MonoEntryHash, im::HashSet<MonoAspectHash>>> {
        let space = self
            .get_space(space_hash)
            .expect("space should already exist");

        let mut out: im::HashMap<
            MonoAgentId,
            im::HashMap<MonoEntryHash, im::HashSet<MonoAspectHash>>,
        > = im::HashMap::new();
        for (entry_hash, entry) in space.all_aspects.iter() {
            for aspect in entry.aspects.iter() {
                for (agent_id, _) in space.connections.iter() {
                    if let Some(set) = space.holding.get(aspect) {
                        if set.contains(agent_id) {
                            continue;
                        }
                    }
                    out.entry(agent_id.clone())
                        .or_default()
                        .entry(entry_hash.clone())
                        .or_default()
                        .insert(aspect.clone());
                }
            }
        }
        out
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
        self.send_mut.send(StoreProto::GetClone(sender)).unwrap();
        StoreRef(receiver.await.unwrap())
    }

    pub fn new_connection(&self, space_hash: SpaceHash, agent_id: AgentId, uri: Lib3hUri) {
        let msg = StoreProto::Mutate(AolEntry::NewConnection {
            aol_idx: self.con_incr.inc(),
            space_hash,
            agent_id,
            uri,
        });
        self.send_mut.send(msg).unwrap();
    }

    pub fn drop_connection(&self, space_hash: SpaceHash, agent_id: AgentId) {
        self.send_mut
            .send(StoreProto::Mutate(AolEntry::DropConnection {
                aol_idx: self.con_incr.inc(),
                space_hash,
                agent_id,
            }))
            .unwrap();
    }

    pub fn drop_connection_by_uri(&self, uri: Lib3hUri) {
        self.send_mut
            .send(StoreProto::Mutate(AolEntry::DropConnectionByUri {
                aol_idx: self.con_incr.inc(),
                uri,
            }))
            .unwrap();
    }

    pub fn agent_holds_aspects(
        &self,
        space_hash: SpaceHash,
        agent_id: AgentId,
        entry_hash: EntryHash,
        aspects: im::HashSet<AspectHash>,
    ) {
        self.send_mut
            .send(StoreProto::Mutate(AolEntry::AgentHoldsAspects {
                aol_idx: self.con_incr.inc(),
                space_hash,
                agent_id,
                entry_hash,
                aspects,
            }))
            .unwrap();
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

        let store = Store::new(crypto);

        //store.add_aspect(space_hash.clone(), "test".into(), im::hashset! {"one".into(), "two".into()});

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
        println!(
            "{:#?}",
            store
                .get_clone()
                .await
                .get_agents_missing_aspects(&space_hash),
        );
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
