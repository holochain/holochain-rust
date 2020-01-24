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

pub type AspectHash = String;
pub type EntryHash = String;
pub type AgentId = String;

/// Append-Only-Log Entries for mutating the Sim2h Store
/// with a list of these, we should be able to reconstruct the store
/// even if they come out-of-order.
#[derive(Debug)]
enum AolEntry {
    // we now know this entry/aspects exists
    // - add it to `all_aspects`
    AddAspects {
        entry_addr: EntryHash,
        aspects: im::HashSet<AspectHash>,
    },

    // all we know is this agent MAY be connected (if con_incr is > cur)
    // - set connections entry to is_connected=true
    // - add entry to `uri_to_connection`
    // - clear `holding`?
    NewConnection {
        con_incr: u64,
        agent_id: AgentId,
        agent_loc: u32,
        uri: String,
    },

    // we will no longer rely on this agent/connection (if con_incr is > cur)
    // - mark connection as disconnected (tombstone)
    // - clear all `holding` aspects (to prepare for another connection
    // - remove the uri_to_connection entry
    DropConnection {
        con_incr: u64,
        agent_id: AgentId,
    },

    // if this agent is currently assumed connected (&& con_incr is > cur)
    // mark that they are likely `holding` these aspects
    AgentHoldsAspects {
        con_incr: u64,
        agent_id: AgentId,
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
    con_incr: u64,
    agent_loc: u32,
    uri: String,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            con_incr: 0,
            agent_loc: 0,
            uri: "".into(),
        }
    }
}

/// so we cache entry locations as well
#[derive(Debug, Clone)]
pub struct Entry {
    entry_loc: u32,
    aspects: im::HashSet<AspectHash>,
}

/// sim2h state storage
#[derive(Debug, Clone)]
pub struct Store {
    pub all_aspects: im::HashMap<EntryHash, Entry>,
    pub connections: im::HashMap<AgentId, ConnectionState>,
    pub uri_to_connection: im::HashMap<String, AgentId>,
    pub holding: im::HashMap<AspectHash, im::HashSet<AgentId>>,
    pub con_incr: Arc<AtomicU64>,
}

impl Store {
    pub fn new() -> StoreHandle {
        let (send_mut, mut recv_mut) = tokio::sync::mpsc::unbounded_channel();

        let ref_dummy = Arc::new(());

        let con_incr = Arc::new(AtomicU64::new(1));
        let mut store = Store {
            all_aspects: im::HashMap::new(),
            connections: im::HashMap::new(),
            uri_to_connection: im::HashMap::new(),
            holding: im::HashMap::new(),
            con_incr: con_incr.clone(),
        };

        let weak_ref_dummy = Arc::downgrade(&ref_dummy);

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
            AolEntry::AddAspects {
                entry_addr,
                aspects,
            } => self.add_aspect(entry_addr, aspects),
            AolEntry::NewConnection {
                con_incr,
                agent_id,
                agent_loc,
                uri,
            } => self.new_connection(con_incr, agent_id, agent_loc, uri),
            AolEntry::DropConnection { con_incr, agent_id } => {
                self.drop_connection(con_incr, agent_id)
            }
            AolEntry::AgentHoldsAspects {
                con_incr,
                agent_id,
                aspects,
            } => self.agent_holds_aspects(con_incr, agent_id, aspects),
        }
    }

    fn add_aspect(&mut self, entry_addr: EntryHash, aspects: im::HashSet<AspectHash>) {
        let e = self.all_aspects.entry(entry_addr).or_insert_with(|| {
            Entry {
                entry_loc: 0, // <- TODO actually calculate this
                aspects: im::HashSet::new(),
            }
        });

        for a in aspects {
            e.aspects.insert(a);
        }
    }

    fn new_connection(&mut self, con_incr: u64, agent_id: AgentId, agent_loc: u32, uri: String) {
        // - set connections entry to is_connected=true
        let c = self.connections.entry(agent_id.clone()).or_default();
        if c.con_incr >= con_incr {
            return;
        }
        c.con_incr = con_incr;
        c.agent_loc = agent_loc;
        c.uri = uri;
        // - add entry to `uri_to_connection`
        self.uri_to_connection.insert(c.uri.clone(), agent_id);
        // - TODO clear `holding`?
    }

    fn drop_connection(&mut self, con_incr: u64, agent_id: AgentId) {
        // - mark connection as disconnected (tombstone)
        let uri = match self.connections.entry(agent_id.clone()) {
            im::hashmap::Entry::Occupied(entry) => {
                if entry.get().con_incr >= con_incr {
                    return;
                }
                entry.remove().uri
            }
            _ => return,
        };
        // - remove the uri_to_connection entry
        self.uri_to_connection.remove(&uri);
        // - clear all `holding` aspects (to prepare for another connection
        for h in self.holding.iter_mut() {
            h.remove(&agent_id);
        }
    }

    fn agent_holds_aspects(
        &mut self,
        con_incr: u64,
        agent_id: AgentId,
        aspects: im::HashSet<AspectHash>,
    ) {
        match self.connections.entry(agent_id.clone()) {
            im::hashmap::Entry::Occupied(entry) => {
                if entry.get().con_incr >= con_incr {
                    return;
                }
            }
            _ => return,
        }
        for aspect in aspects {
            self.holding
                .entry(aspect)
                .or_default()
                .insert(agent_id.clone());
        }
    }

    /// return a mapping of all entry_hash/aspect_hashes
    /// that each agent is missing (note how it returns references :+1:)
    pub fn get_agents_missing_aspects(
        &self,
    ) -> im::HashMap<&AgentId, im::HashMap<&EntryHash, im::HashSet<&AspectHash>>> {
        let mut out: im::HashMap<&AgentId, im::HashMap<&EntryHash, im::HashSet<&AspectHash>>> =
            im::HashMap::new();
        for (entry_hash, entry) in self.all_aspects.iter() {
            for aspect in entry.aspects.iter() {
                for (agent_id, _) in self.connections.iter() {
                    if let Some(set) = self.holding.get(aspect) {
                        if set.contains(agent_id) {
                            continue;
                        }
                    }
                    out.entry(agent_id)
                        .or_default()
                        .entry(entry_hash)
                        .or_default()
                        .insert(aspect);
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

    pub fn add_aspect(&self, entry_addr: EntryHash, aspects: im::HashSet<AspectHash>) {
        self.send_mut
            .send(StoreProto::Mutate(AolEntry::AddAspects {
                entry_addr,
                aspects,
            }))
            .unwrap();
    }

    pub fn new_connection(&self, agent_id: AgentId, uri: String) {
        let msg = StoreProto::Mutate(AolEntry::NewConnection {
            con_incr: self.con_incr.inc(),
            agent_id,
            agent_loc: 0, // <- TODO - actually gen loc from agent id
            uri,
        });
        self.send_mut.send(msg).unwrap();
    }

    pub fn drop_connection(&self, agent_id: AgentId) {
        self.send_mut
            .send(StoreProto::Mutate(AolEntry::DropConnection {
                con_incr: self.con_incr.inc(),
                agent_id,
            }))
            .unwrap();
    }

    pub fn agent_holds_aspects(&self, agent_id: AgentId, aspects: im::HashSet<AspectHash>) {
        self.send_mut
            .send(StoreProto::Mutate(AolEntry::AgentHoldsAspects {
                con_incr: self.con_incr.inc(),
                agent_id,
                aspects,
            }))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn async_test_demo_im_state() {
        println!("yo");
        let store = Store::new();

        store.add_aspect("test".into(), im::hashset! {"one".into(), "two".into()});

        println!("GOT: {:#?}", store.get_clone().await);

        store.new_connection("id-1".into(), "ws://yada1".into());
        store.new_connection("id-2".into(), "ws://yada2".into());

        println!("GOT: {:#?}", store.get_clone().await);

        store.agent_holds_aspects("id-1".into(), im::hashset! {"one".into(), "two".into()});
        store.agent_holds_aspects("id-2".into(), im::hashset! {"one".into()});

        println!("GOT: {:#?}", store.get_clone().await);

        println!("--- beg check missing ---");
        println!(
            "{:#?}",
            store.get_clone().await.get_agents_missing_aspects()
        );
        println!("--- end check missing ---");

        store.drop_connection("id-1".into());

        println!("GOT: {:#?}", store.get_clone().await);
    }

    #[test]
    fn demo_im_state() {
        tokio::runtime::Builder::new()
            .threaded_scheduler()
            .core_threads(num_cpus::get())
            .thread_name("tokio-rkv-test-thread")
            .enable_all()
            .build()
            .unwrap()
            .block_on(async_test_demo_im_state());
    }
}
