use action::ActionWrapper;
use agent::{chain_store::ChainStore, state::AgentState};
use context::Context;
use dht::dht_store::DhtStore;
use holochain_cas_implementations::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
use nucleus::state::NucleusState;
use std::{collections::HashSet, sync::Arc};

/// The Store of the Holochain instance Object, according to Redux pattern.
/// It's composed of all sub-module's state slices.
/// To plug in a new module, its state slice needs to be added here.
#[derive(Clone, PartialEq, Debug)]
pub struct State {
    nucleus: Arc<NucleusState>,
    agent: Arc<AgentState>,
    dht: Arc<DhtStore<FilesystemStorage, EavFileStorage>>,
    // @TODO eventually drop stale history
    // @see https://github.com/holochain/holochain-rust/issues/166
    pub history: HashSet<ActionWrapper>,
}

impl State {
    pub fn new(context: Arc<Context>) -> Self {
        // @TODO file table
        // @see https://github.com/holochain/holochain-rust/pull/246

        let cas = &(*context).file_storage;
        let eav = &(*context).eav_storage;
        State {
            nucleus: Arc::new(NucleusState::new()),
            agent: Arc::new(AgentState::new(ChainStore::new(cas.clone()))),
            dht: Arc::new(DhtStore::new(cas.clone(), eav.clone())),
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

    pub fn dht(&self) -> Arc<DhtStore<FilesystemStorage, EavFileStorage>> {
        Arc::clone(&self.dht)
    }
}

pub fn test_store(context: Arc<Context>) -> State {
    State::new(context)
}
