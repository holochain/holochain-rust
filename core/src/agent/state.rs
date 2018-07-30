use agent::keys::Keys;
use chain::Chain;
use hash_table::{entry::Entry, memory::MemTable, pair::Pair};
use instance::Observer;
use snowflake;
use state;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};

#[derive(Clone, Debug, PartialEq, Default)]
/// struct to track the internal state of an agent exposed to reducers/observers
pub struct AgentState {
    keys: Option<Keys>,
    // @TODO how should this work with chains/HTs?
    // @see https://github.com/holochain/holochain-rust/issues/137
    // @see https://github.com/holochain/holochain-rust/issues/135
    top_pair: Option<Pair>,
    /// every action and the result of that action
    // @TODO this will blow up memory, implement as some kind of dropping/FIFO with a limit?
    // @see https://github.com/holochain/holochain-rust/issues/166
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

    /// getter for a copy of self.actions
    /// uniquely maps action executions to the result of the action
    pub fn actions(&self) -> HashMap<Action, ActionResult> {
        self.actions.clone()
    }
}

#[derive(Clone, PartialEq, Hash, Debug)]
/// a single action to perform
/// every action must have a unique id or there will be collisions in AgentState::actions
/// the convenience methods for each action variant generate ids correctly
pub enum Action {
    /// zome API function: commit
    Commit {
        entry: Entry,
        id: snowflake::ProcessUniqueId,
    },
    /// zome API function: get
    Get {
        key: String,
        id: snowflake::ProcessUniqueId,
    },
}

impl Action {
    /// returns a new Action::Commit for the passed entry
    pub fn commit(entry: &Entry) -> Action {
        Action::Commit {
            id: snowflake::ProcessUniqueId::new(),
            entry: entry.clone(),
        }
    }

    /// returns a new Action::Get for the passed key
    pub fn get(key: &str) -> Action {
        Action::Get {
            id: snowflake::ProcessUniqueId::new(),
            key: key.to_string(),
        }
    }
}

impl Eq for Action {}

#[derive(Clone, Debug, PartialEq)]
/// the result of a single action performed
/// stored alongside the action in AgentState::actions to provide a state history that observers
/// poll and retrieve
pub enum ActionResult {
    Commit(String),
    Get(Option<Pair>),
}

/// do a commit action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn do_action_commit(state: &mut AgentState, action: &Action) {
    match action {
        Action::Commit { entry, .. } => {
            // add entry to source chain
            // @TODO this does nothing!
            // it needs to get something stateless from the agent state that points to
            // something stateful that can handle an entire hash table (e.g. actor)
            // @see https://github.com/holochain/holochain-rust/issues/135
            // @see https://github.com/holochain/holochain-rust/issues/148
            let mut chain = Chain::new(Rc::new(MemTable::new()));

            // @TODO successfully validate before pushing a commit
            // @see https://github.com/holochain/holochain-rust/issues/97

            let result = chain.push(&entry).unwrap().entry().key();
            state
                .actions
                .insert(action.clone(), ActionResult::Commit(result));
        }
        _ => {
            panic!("action commit without commit action");
        }
    }
}

/// do a get action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn do_action_get(state: &mut AgentState, action: &Action) {
    match action {
        Action::Get { key, .. } => {
            // get pair from source chain
            // @TODO this does nothing!
            // it needs to get something stateless from the agent state that points to
            // something stateful that can handle an entire hash table (e.g. actor)
            // @see https://github.com/holochain/holochain-rust/issues/135
            // @see https://github.com/holochain/holochain-rust/issues/148

            // drop in a dummy entry for testing
            let mut chain = Chain::new(Rc::new(MemTable::new()));
            let e = Entry::new("testEntryType", "test entry content");
            chain.push(&e).unwrap();

            // @TODO if the get fails local, do a network get
            // @see https://github.com/holochain/holochain-rust/issues/167

            let result = chain.get_entry(&key).unwrap();
            state
                .actions
                .insert(action.clone(), ActionResult::Get(result));
        }
        _ => {
            panic!("action get without get action");
        }
    }
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
                ref action @ Action::Commit { .. } => {
                    do_action_commit(&mut new_state, &action);
                }
                ref action @ Action::Get { .. } => {
                    do_action_get(&mut new_state, &action);
                }
            }
            Arc::new(new_state)
        }
        _ => old_state,
    }
}

#[cfg(test)]
pub mod tests {
    use super::{do_action_commit, do_action_get, Action, ActionResult, AgentState};
    use hash_table::{entry::tests::test_entry, pair::tests::test_pair};
    use std::collections::HashMap;

    /// builds a dummy agent state for testing
    pub fn test_agent_state() -> AgentState {
        AgentState::new()
    }

    /// builds a dummy action for testing commit
    pub fn test_action_commit() -> Action {
        Action::commit(&test_entry())
    }

    /// builds a dummy action result for testing commit
    pub fn test_action_result_commit() -> ActionResult {
        ActionResult::Commit(test_entry().key())
    }

    /// builds a dummy action for testing get
    pub fn test_action_get() -> Action {
        Action::get(&test_entry().key())
    }

    /// builds a dummy action result for testing get
    pub fn test_action_result_get() -> ActionResult {
        ActionResult::Get(Some(test_pair()))
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

    #[test]
    /// test for the agent state actions getter
    fn agent_state_actions() {
        assert_eq!(HashMap::new(), test_agent_state().actions());
    }

    #[test]
    /// smoke test building a new commit action + result
    fn action_commit() {
        test_action_commit();
        test_action_result_commit();

        // actions have unique ids and are not equal
        assert_ne!(test_action_commit(), test_action_commit());
        // the result is equal though
        assert_eq!(test_action_result_commit(), test_action_result_commit());
    }

    #[test]
    /// smoke test building a new get action + result
    fn action_get() {
        test_action_get();
        test_action_result_get();

        // actions have unique ids and are not equal
        assert_ne!(test_action_get(), test_action_get());
        // the result is equal though
        assert_eq!(test_action_result_get(), test_action_result_get());
    }

    #[test]
    /// test for action commit
    fn agent_state_do_commit() {
        let mut state = test_agent_state();
        let action = test_action_commit();

        do_action_commit(&mut state, &action);

        assert_eq!(
            state.actions().get(&action),
            Some(&test_action_result_commit()),
        );
    }

    #[test]
    /// test for action get
    fn agent_state_do_get() {
        let mut state = test_agent_state();
        let action = test_action_get();

        do_action_get(&mut state, &action);

        assert_eq!(
            state.actions().get(&action),
            Some(&test_action_result_get()),
        );
    }
}
