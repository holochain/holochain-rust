use agent::keys::Keys;
use chain::Chain;
use hash_table::{entry::Entry, memory::MemTable, pair::Pair};
use instance::Observer;
// use snowflake;
use state;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};
use action::Action;
use action::ActionResult;
use action::commit::Commit;
use action::commit::CommitResult;
use action::get::Get;

enum ActionHistory {
    Key(Action),
    Value(ActionResult),
}

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
    // commits: HashMap<Commit, CommitResult>,
    actions: HashMap<ActionHistory::Key, ActionHistory::Value>,
    // gets: HashMap<Get, GetResult>,
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
    pub fn actions<A: Action, AR: ActionResult>(&self) -> HashMap<A, AR> {
        self.actions.clone()
    }
}

// #[derive(Clone, PartialEq, Hash, Debug)]
/// a single action to perform
/// every action must have a unique id or there will be collisions in AgentState::actions
/// the convenience methods for each action variant generate ids correctly
// pub enum Action {
//     /// zome API function: commit
//     Commit {
//         entry: Entry,
//         id: snowflake::ProcessUniqueId,
//     },
//     /// zome API function: get
//     Get {
//         key: String,
//         id: snowflake::ProcessUniqueId,
//     },
// }
//
// impl Action {
//     /// returns a new Action::Commit for the passed entry
//     pub fn commit(entry: &Entry) -> Action {
//         Action::Commit {
//             id: snowflake::ProcessUniqueId::new(),
//             entry: entry.clone(),
//         }
//     }
//
//     /// returns a new Action::Get for the passed key
//     pub fn get(key: &str) -> Action {
//         Action::Get {
//             id: snowflake::ProcessUniqueId::new(),
//             key: key.to_string(),
//         }
//     }
// }

// impl Eq for Action {}

// #[derive(Clone, Debug, PartialEq)]
/// the result of a single action performed
/// stored alongside the action in AgentState::actions to provide a state history that observers
/// poll and retrieve
// pub enum ActionResult {
//     Commit(String),
//     Get(Option<Pair>),
// }

/// do a commit action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn handle_commit (state: &mut AgentState, commit: &Commit) {
    // add entry to source chain
    // @TODO this does nothing!
    // it needs to get something stateless from the agent state that points to
    // something stateful that can handle an entire hash table (e.g. actor)
    // @see https://github.com/holochain/holochain-rust/issues/135
    // @see https://github.com/holochain/holochain-rust/issues/148
    let mut chain = Chain::new(Rc::new(MemTable::new()));

    // @TODO successfully validate before pushing a commit
    // @see https://github.com/holochain/holochain-rust/issues/97

    let result = chain.push(&commit.entry).unwrap().entry().key();
    state
        .commits
        .insert(commit.clone(), CommitResult::new(result));
}

/// do a get action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn handle_get (state: &mut AgentState, action: &Get) {
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

fn resolve_action_handler<A: Action>(action: &A) -> Option<fn(&mut AgentState, &A)> {
    match action {
        Commit => Some(handle_commit),
        Get => Some(handle_get),
        _ => None,
    }
}

/// Reduce Agent's state according to provided Action
pub fn reduce<A: Action>(
    old_state: Arc<AgentState>,
    action: &A,
    _action_channel: &Sender<state::ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) -> Arc<AgentState> {
    let handler = resolve_action_handler(action);
    match handler {
        Some(f) => {
            let mut new_state: AgentState = (*old_state).clone();
            f(&mut new_state, &action);
            Arc::new(new_state)
        },
        None => old_state,
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
