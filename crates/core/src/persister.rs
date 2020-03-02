use crate::{
    agent::state::{AgentStateSnapshot, AGENT_SNAPSHOT_ADDRESS},
    context::Context,
    nucleus::state::{NucleusStateSnapshot, NUCLEUS_SNAPSHOT_ADDRESS},
    state::State,
    
};
use holochain_core_types::error::HolochainError;
use holochain_locksmith::RwLock;

use holochain_persistence_api::cas::{
    content::{Address, AddressableContent, Content},
    storage::ContentAddressableStorage,
};

use crate::{
    dht::dht_store::{DhtStoreSnapshot, DHT_STORE_SNAPSHOT_ADDRESS},
    state::StateWrapper,
};
use std::sync::Arc;

/// trait that defines the persistence functionality that holochain_core requires
pub trait Persister: Send + Sync {
    // @TODO how does save/load work with snowflake IDs?
    // snowflake is only unique across a single process, not a reboot save/load round trip
    // we'd need real UUIDs for persistant uniqueness
    // @see https://github.com/holochain/holochain-rust/issues/203
    fn save(&mut self, state: &StateWrapper) -> Result<(), HolochainError>;
    fn load(&self, context: Arc<Context>) -> Result<Option<State>, HolochainError>;
}

#[derive(Clone)]
pub struct SimplePersister {
    storage: Arc<RwLock<dyn ContentAddressableStorage>>,
}

impl PartialEq for SimplePersister {
    fn eq(&self, other: &SimplePersister) -> bool {
        (&*self.storage.read().unwrap()).get_id() == (&*other.storage.read().unwrap()).get_id()
    }
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Persister for SimplePersister {
    fn save(&mut self, state: &StateWrapper) -> Result<(), HolochainError> {
        let lock = &*self.storage.clone();
        let mut store = lock.write()?;
        let agent_snapshot = AgentStateSnapshot::from(state);
        let nucleus_snapshot = NucleusStateSnapshot::from(state);
        let dht_store_snapshot = DhtStoreSnapshot::from(state);
        store.add(&agent_snapshot)?;
        store.add(&nucleus_snapshot)?;
        store.add(&dht_store_snapshot)?;
        Ok(())
    }
    fn load(&self, context: Arc<Context>) -> Result<Option<State>, HolochainError> {
        let lock = &*self.storage.clone();
        let store = lock.read().unwrap();

        let agent_snapshot: Option<AgentStateSnapshot> = store
            .fetch(&Address::from(AGENT_SNAPSHOT_ADDRESS))?
            .map(|s: Content| {
                AgentStateSnapshot::try_from_content(&s)
                    .expect("could not load AgentStateSnapshot from content")
            });

        let nucleus_snapshot: Option<NucleusStateSnapshot> = store
            .fetch(&Address::from(NUCLEUS_SNAPSHOT_ADDRESS))?
            .map(|s: Content| {
                NucleusStateSnapshot::try_from_content(&s)
                    .expect("could not load NucleusStateSnapshot from content")
            });

        let dht_store_snapshot: Option<DhtStoreSnapshot> = store
            .fetch(&Address::from(DHT_STORE_SNAPSHOT_ADDRESS))?
            .map(|s: Content| {
                DhtStoreSnapshot::try_from_content(&s)
                    .expect("could not load DhtStoreSnapshot from content")
            });

        if agent_snapshot.is_none() || nucleus_snapshot.is_none() || dht_store_snapshot.is_none() {
            return Ok(None);
        }

        Ok(State::try_from_snapshots(
            context,
            agent_snapshot.unwrap(),
            nucleus_snapshot.unwrap(),
            dht_store_snapshot.unwrap(),
        )
        .ok())
    }
}

impl SimplePersister {
    pub fn new(storage: Arc<RwLock<dyn ContentAddressableStorage>>) -> Self {
        SimplePersister { storage }
    }
}

#[cfg(test)]
mod tests {

    use self::tempfile::tempdir;
    use crate::{
        instance::tests::test_context_with_agent_state,
        persister::{Persister, SimplePersister},
    };
    use std::fs::File;
    use tempfile;

    #[test]
    fn persistence_round_trip() {
        let dir = tempdir().unwrap();
        let temp_path = dir.path().join("test");
        let _tempfile = temp_path.to_str().unwrap();
        let context = test_context_with_agent_state(None);
        File::create(temp_path.clone()).unwrap();
        let mut persistance = SimplePersister::new(context.dht_storage.clone());
        let state = context.state().unwrap().clone();
        persistance.save(&state).unwrap();
        let state_from_file = persistance.load(context).unwrap().unwrap();
        assert_eq!(state.agent(), state_from_file.agent());
        assert_eq!(state.nucleus(), state_from_file.nucleus());
        assert_eq!(state.dht(), state_from_file.dht());

        // the network is NOT the same because it can't be serialzied rationally
        // need to fix this so `persitance.load()` takes a networks or something
        assert_ne!(state.network(), state_from_file.network());
    }
}
