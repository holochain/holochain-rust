use action::{Action, ActionWrapper, Signal};
use agent::keys::Keys;
use chain::Chain;
use hash_table::{entry::Entry, memory::MemTable, pair::Pair};
use instance::Observer;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};
use std::thread;
use nucleus::ribosome::lifecycle::validate_commit::validate_commit;
use nucleus::ribosome::lifecycle::LifecycleFunctionParams;

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
    actions: HashMap<Action, ActionResponse>,
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
    pub fn actions(&self) -> HashMap<Action, ActionResponse> {
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

#[derive(Clone, Debug, PartialEq)]
/// the agent's response to an action
/// stored alongside the action in AgentState::actions to provide a state history that observers
/// poll and retrieve
pub enum ActionResponse {
    Commit(String),
    Get(Option<Pair>),
}

/// do a commit action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn handle_commit(
    state: &mut AgentState,
    action: &Action,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) {
    let signal = action.signal();
    let (function_call, entry) = match signal {
        Signal::Commit(r, e) => (r, e),
        _ => unreachable!(),
    };

    // add entry to source chain
    // @TODO this does nothing!
    // it needs to get something stateless from the agent state that points to
    // something stateful that can handle an entire hash table (e.g. actor)
    // @see https://github.com/holochain/holochain-rust/issues/135
    // @see https://github.com/holochain/holochain-rust/issues/148
    let mut chain = Chain::new(Rc::new(MemTable::new()));

    let validate_action_channel = action_channel.clone();
    let validate_observer_channel = observer_channel.clone();
    let validate_entry = entry.clone();
    thread::spawn(move || {
        validate_commit(
            &validate_action_channel,
            &validate_observer_channel,
            &function_call.zome,
            LifecycleFunctionParams::ValidateCommit(validate_entry),
        );
    });

    let result = chain.push(&entry).unwrap().entry().key();
    state
        .actions
        .insert(action.clone(), ActionResponse::Commit(result.clone()));
}

/// do a get action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn handle_get(
    state: &mut AgentState,
    action: &Action,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let signal = action.signal();
    let key = unwrap_to!(signal => Signal::Get);

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
        .insert(action.clone(), ActionResponse::Get(result.clone()));
}

fn resolve_action_handler(action: &Action)
    -> Option<fn(&mut AgentState, &Action, &Sender<ActionWrapper>, &Sender<Observer>,)> {
    match action.signal() {
        Signal::Commit(_, _) => Some(handle_commit),
        Signal::Get(_) => Some(handle_get),
        _ => None,
    }
}

/// Reduce Agent's state according to provided Action
pub fn reduce(
    old_state: Arc<AgentState>,
    action: &Action,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) -> Arc<AgentState> {
    let handler = resolve_action_handler(action);
    match handler {
        Some(f) => {
            let mut new_state: AgentState = (*old_state).clone();
            f(
                &mut new_state,
                &action,
                action_channel,
                observer_channel,
            );
            Arc::new(new_state)
        }
        None => old_state,
    }
}

#[cfg(test)]
pub mod tests {
    use super::{handle_commit, handle_get, ActionResponse, AgentState};
    use action::{Action, Signal};
    use hash::tests::test_hash;
    use hash_table::{pair::tests::test_pair};
    use std::collections::HashMap;
    use action::tests::test_action_commit;
    use instance::tests::test_instance_blank;

    /// builds a dummy agent state for testing
    pub fn test_agent_state() -> AgentState {
        AgentState::new()
    }

    pub fn test_action_response_commit() -> ActionResponse {
        ActionResponse::Commit(test_hash())
    }

    pub fn test_action_response_get() -> ActionResponse {
        ActionResponse::Get(Some(test_pair()))
    }

    pub fn test_action_get() -> Action {
        Action::new(&Signal::Get(test_hash()))
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
    fn agent_state_handle_commit() {
        let mut state = test_agent_state();
        let action = test_action_commit();

        let instance = test_instance_blank();

        handle_commit(
            &mut state,
            &action,
            &instance.action_channel(),
            &instance.observer_channel(),
        );

        assert_eq!(
            state.actions().get(&action),
            Some(&test_action_response_commit()),
        );
    }

    #[test]
    /// test for action get
    fn agent_state_handle_get() {
        let mut state = test_agent_state();
        let action = test_action_get();

        let instance = test_instance_blank();

        handle_get(
            &mut state,
            &action,
            &instance.action_channel(),
            &instance.observer_channel(),
        );

        assert_eq!(
            state.actions().get(&action),
            Some(&test_action_response_get()),
        );
    }
}
