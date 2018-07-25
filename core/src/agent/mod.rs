pub mod keys;

use instance::Observer;
use agent::keys::Keys;
use chain::Chain;
use snowflake;
use hash_table::{entry::Entry, memory::MemTable, pair::Pair};
use state;
use std::{
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AgentState {
    keys: Option<Keys>,
    // @TODO how should this work with chains/HTs?
    // @see https://github.com/holochain/holochain-rust/issues/137
    // @see https://github.com/holochain/holochain-rust/issues/135
    top_pair: Option<Pair>,
    // @TODO this will blow up memory, implement as some kind of dropping/FIFO with a limit?
    actions: HashMap<Action, ActionResult>,
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new() -> AgentState {
        AgentState {
            keys: None,
            top_pair: None,
            actions: HashMap::new(),
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

    pub fn actions(&self) -> HashMap<Action, ActionResult> {
        self.actions.clone()
    }
}

#[derive(Clone, PartialEq, Hash, Debug)]
pub enum Action {
    Commit(Entry),
    Get {
        key: String,
        id: snowflake::ProcessUniqueId,
    },
}

impl Eq for Action {}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionResult {
    Commit(String),
    Get(Option<Pair>),
}

/// Reduce Agent's state according to provided Action
pub fn reduce(
    old_state: Arc<AgentState>,
    action: &state::Action,
    _action_channel: &Sender<state::ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) -> Arc<AgentState> {
    match *action {
        state::Action::Agent(ref agent_action) => {
            let mut new_state: AgentState = (*old_state).clone();
            match *agent_action {
                Action::Commit(ref entry) => {
                    // add entry to source chain
                    // @TODO this does nothing!
                    // it needs to get something stateless from the agent state that points to
                    // something stateful that can handle an entire hash table (e.g. actor)
                    // @see https://github.com/holochain/holochain-rust/issues/135
                    // @see https://github.com/holochain/holochain-rust/issues/148
                    let mut chain = Chain::new(Rc::new(MemTable::new()));
                    chain.push(&entry).unwrap();

                    let result = chain.push(&entry).unwrap().key();
                    new_state.actions.insert(
                        agent_action.clone(),
                        ActionResult::Commit(result),
                    );
                },
                Action::Get{ ref key, id: _ } => {
                    // get pair from source chain
                    // @TODO this does nothing!
                    // it needs to get something stateless from the agent state that points to
                    // something stateful that can handle an entire hash table (e.g. actor)
                    // @see https://github.com/holochain/holochain-rust/issues/135
                    // @see https://github.com/holochain/holochain-rust/issues/148

                    // drop in a dummy entry for testing
                    let mut chain = Chain::new(Rc::new(MemTable::new()));
                    let e = Entry::new("fake entry type", "fake entry content");
                    chain.push(&e).unwrap();

                    let result = chain.get(&key).unwrap();
                    new_state.actions.insert(
                        agent_action.clone(),
                        ActionResult::Get(result),
                    );
                },
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
