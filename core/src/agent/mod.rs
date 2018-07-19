pub mod keys;

use agent::keys::Keys;
use chain::Chain;
use hash_table::{entry::Entry, memory::MemTable, pair::Pair};
use state;
use std::{
    rc::Rc, sync::{mpsc::Sender, Arc},
};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AgentState {
    keys: Option<Keys>,
    // @TODO how should this work with chains/HTs?
    // @see https://github.com/holochain/holochain-rust/issues/137
    // @see https://github.com/holochain/holochain-rust/issues/135
    top_pair: Option<Pair>,
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new() -> AgentState {
        AgentState {
            keys: None,
            top_pair: None,
        }
    }

    /// getter for a copy of self.keys
    pub fn keys(&self) -> Option<Keys> {
        self.keys.clone()
    }

    /// getter for a copy of self.top_pair
    /// should be used with a source chain for validation/safety
    pub fn top_pair(&self) -> Option<Pair> {
        self.top_pair.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Commit(Entry),
}

/// Reduce Agent's state according to provided Action
pub fn reduce(
    old_state: Arc<AgentState>,
    action: &state::Action,
    _action_channel: &Sender<state::ActionWrapper>,
) -> Arc<AgentState> {
    match *action {
        state::Action::Agent(ref agent_action) => {
            let mut new_state: AgentState = (*old_state).clone();
            match *agent_action {
                Action::Commit(ref entry) => {
                    // add entry to source chain
                    // @TODO this does nothing! it isn't exactly clear what it should do either
                    // @see https://github.com/holochain/holochain-rust/issues/148
                    let mut chain = Chain::new(Rc::new(MemTable::new()));
                    chain.push(&entry).unwrap();
                }
            }
            Arc::new(new_state)
        }
        _ => old_state,
    }
}

#[cfg(test)]
pub mod tests {
    use super::AgentState;

    /// builds a dummy agent state for testing
    pub fn test_agent_state() -> AgentState {
        AgentState::new()
    }

    #[test]
    /// smoke test for building a new AgentState
    fn agent_state_new() {
        test_agent_state();
    }

    #[test]
    /// test for the agent state keys getter
    fn agent_state_keys() {
        assert_eq!(None, test_agent_state().keys());
    }

    #[test]
    /// test for the agent state top pair getter
    fn agent_state_top_pair() {
        assert_eq!(None, test_agent_state().top_pair());
    }
}
