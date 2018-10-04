use action::ActionWrapper;
use agent::state::AgentState;
use cas::memory::MemoryStorage;
use chain::Chain;
use context::Context;
use dht::dht_store::DhtStore;
use eav::memory::EavMemoryStorage;
use hash_table::{actor::HashTableActor, memory::MemTable};
use nucleus::state::NucleusState;
use std::{collections::HashSet, sync::Arc};

/// The Store of the Holochain instance Object, according to Redux pattern.
/// It's composed of all sub-module's state slices.
/// To plug in a new module, its state slice needs to be added here.
#[derive(Clone, PartialEq, Debug)]
pub struct State {
    nucleus: Arc<NucleusState>,
    agent: Arc<AgentState>,
    dht: Arc<DhtStore<MemoryStorage, EavMemoryStorage>>,
    // @TODO eventually drop stale history
    // @see https://github.com/holochain/holochain-rust/issues/166
    pub history: HashSet<ActionWrapper>,
}

impl State {
    pub fn new() -> Self {
        // @TODO file table
        // @see https://github.com/holochain/holochain-rust/pull/246
        let chain = Chain::new(HashTableActor::new_ref(MemTable::new()));

        let content_storage = MemoryStorage::new();
        let eav_storage = EavMemoryStorage::new();

        State {
            nucleus: Arc::new(NucleusState::new()),
            agent: Arc::new(AgentState::new(&chain)),
            dht: Arc::new(DhtStore::new(content_storage, eav_storage)),
            history: HashSet::new(),
        }
    }

    pub fn reduce(&self, context: Arc<Context>, action_wrapper: ActionWrapper) -> Self {
        let mut new_state = State {
            nucleus: ::nucleus::reduce(
                Arc::clone(&context),
                Arc::clone(&self.nucleus),
                &action_wrapper,
            ),
            agent: ::agent::state::reduce(
                Arc::clone(&context),
                Arc::clone(&self.agent),
                &action_wrapper,
            ),
            dht: ::dht::dht_reducers::reduce(
                Arc::clone(&context),
                Arc::clone(&self.dht),
                &action_wrapper,
            ),
            history: self.history.clone(),
        };

        new_state.history.insert(action_wrapper);
        new_state
    }

    pub fn nucleus(&self) -> Arc<NucleusState> {
        Arc::clone(&self.nucleus)
    }

    pub fn agent(&self) -> Arc<AgentState> {
        Arc::clone(&self.agent)
    }

    pub fn dht(&self) -> Arc<DhtStore<MemoryStorage, EavMemoryStorage>> {
        Arc::clone(&self.dht)
    }
}
