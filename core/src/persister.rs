use agent::state::{AgentStateSnapshot, AGENT_SNAPSHOT_ADDRESS};
use context::Context;
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    error::HolochainError,
};
use state::State;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    sync::{Arc, RwLock},
};

/// trait that defines the persistence functionality that holochain_core requires
pub trait Persister: Send {
    // @TODO how does save/load work with snowflake IDs?
    // snowflake is only unique across a single process, not a reboot save/load round trip
    // we'd need real UUIDs for persistant uniqueness
    // @see https://github.com/holochain/holochain-rust/issues/203
    fn save(&mut self, state: State) -> Result<(), HolochainError>;
    fn load(&self, context: Arc<Context>) -> Result<Option<State>, HolochainError>;
}

#[derive(Clone)]
pub struct SimplePersister {
    storage: Arc<RwLock<ContentAddressableStorage>>,
}

impl PartialEq for SimplePersister {
    fn eq(&self, other: &SimplePersister) -> bool {
        (&*self.storage.read().unwrap()).get_id() == (&*other.storage.read().unwrap()).get_id()
    }
}

impl Persister for SimplePersister {
    fn save(&mut self, state: State) -> Result<(), HolochainError> {
        let lock = &*self.storage.clone();
        let mut store = lock.write().unwrap();
        let snapshot = State::to_agent_snapshot(state)?;
        Ok(store.add(&snapshot)?)
    }
    fn load(&self, context: Arc<Context>) -> Result<Option<State>, HolochainError> {
        let lock = &*self.storage.clone();
        let mut store = lock.write().unwrap();
        let address = Address::from(AGENT_SNAPSHOT_ADDRESS);
        let snapshot: Option<AgentStateSnapshot> = store
            .fetch(&address)?
            .map(|s: Content| AgentStateSnapshot::from_content(&s));
        let state = snapshot.map(|snap| State::from_agent_snapshot(context, snap).ok());
        Ok(state.unwrap_or(None))
    }
}

impl SimplePersister {
    pub fn new(storage: Arc<RwLock<ContentAddressableStorage>>) -> Self {
        SimplePersister { storage: storage }
    }
}

#[cfg(test)]
mod tests {
    extern crate tempfile;
    use self::tempfile::tempdir;
    use super::*;
    use instance::tests::test_context_with_agent_state;
    #[test]
    fn persistance_round_trip() {
        let dir = tempdir().unwrap();
        let temp_path = dir.path().join("test");
        let tempfile = temp_path.to_str().unwrap();
        let context = test_context_with_agent_state();
        File::create(temp_path.clone()).unwrap();
        let mut persistance = SimplePersister::new(context.file_storage.clone());
        let state = context.state().unwrap().clone();
        persistance.save(state.clone()).unwrap();
        let state_from_file = persistance.load(context).unwrap().unwrap();
        assert_eq!(state, state_from_file)
    }
}
