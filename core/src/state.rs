use action::ActionWrapper;
use agent::state::AgentState;
use chain::Chain;
use context::Context;
use hash_table::{actor::HashTableActor, memory::MemTable};
use instance::Observer;
use nucleus::state::NucleusState;
use std::{
    collections::HashSet,
    sync::{mpsc::Sender, Arc},
};

/// The Store of the Holochain instance Object, according to Redux pattern.
/// Its composed of a Nucleus and Agent state slice.
#[derive(Clone, PartialEq, Debug)]
pub struct State {
    nucleus: Arc<NucleusState>,
    agent: Arc<AgentState>,
    // @TODO eventually drop stale history
    // @see https://github.com/holochain/holochain-rust/issues/166
    pub history: HashSet<ActionWrapper>,
}

impl State {
    pub fn new() -> Self {
        // @TODO file table
        // @see https://github.com/holochain/holochain-rust/pull/246
        let chain = Chain::new(HashTableActor::new_ref(MemTable::new()));

        State {
            nucleus: Arc::new(NucleusState::new()),
            agent: Arc::new(AgentState::new(&chain)),
            history: HashSet::new(),
        }
    }

    pub fn reduce(
        &self,
        context: Arc<Context>,
        action_wrapper: ActionWrapper,
        action_channel: &Sender<ActionWrapper>,
        observer_channel: &Sender<Observer>,
    ) -> Self {
        let mut new_state = State {
            nucleus: ::nucleus::reduce(
                Arc::clone(&context),
                Arc::clone(&self.nucleus),
                &action_wrapper,
                action_channel,
                observer_channel,
            ),
            agent: ::agent::state::reduce(
                Arc::clone(&context),
                Arc::clone(&self.agent),
                &action_wrapper,
                action_channel,
                observer_channel,
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
}
