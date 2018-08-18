use agent::state::AgentState;
use context::Context;
use instance::Observer;
use nucleus::state::NucleusState;
use std::{
    collections::HashSet,
    sync::{mpsc::Sender, Arc},
};
use hash_table::memory::MemTable;
use action::ActionWrapper;
use chain::Chain;
use chain::actor::ChainActor;
use hash_table::actor::HashTableActor;

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
        let chain_actor = ChainActor::new_ref(
            Chain::new(
                HashTableActor::new_ref(
                    MemTable::new(),
                ),
            ),
        );

        State {
            nucleus: Arc::new(NucleusState::new()),
            agent: Arc::new(AgentState::new(chain_actor)),
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
                context.clone(),
                Arc::clone(&self.nucleus),
                &action_wrapper,
                action_channel,
                observer_channel,
            ),
            agent: ::agent::state::reduce(
                context.clone(),
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
