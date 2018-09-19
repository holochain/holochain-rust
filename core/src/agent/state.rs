use action::{Action, ActionWrapper, AgentReduceFn};
use agent::keys::Keys;
use chain::{Chain, SourceChain};
use context::Context;
use error::HolochainError;
use hash::HashString;
use hash_table::entry::Entry;
use instance::Observer;
use json::ToJson;
use key::Key;
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc},
};

/// The state-slice for the Agent.
/// Holds the agent's source chain and keys.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentState {
    keys: Option<Keys>,
    /// every action and the result of that action
    // @TODO this will blow up memory, implement as some kind of dropping/FIFO with a limit?
    // @see https://github.com/holochain/holochain-rust/issues/166
    actions: HashMap<ActionWrapper, ActionResponse>,
    chain: Chain,
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new(chain: &Chain) -> AgentState {
        AgentState {
            keys: None,
            actions: HashMap::new(),
            chain: chain.clone(),
        }
    }

    /// getter for a copy of self.keys
    pub fn keys(&self) -> Option<Keys> {
        self.keys.clone()
    }

    /// getter for the chain
    pub fn chain(&self) -> &Chain {
        &self.chain
    }

    /// getter for a copy of self.actions
    /// uniquely maps action executions to the result of the action
    pub fn actions(&self) -> HashMap<ActionWrapper, ActionResponse> {
        self.actions.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
/// the agent's response to an action
/// stored alongside the action in AgentState::actions to provide a state history that observers
/// poll and retrieve
// @TODO abstract this to a standard trait
// @see https://github.com/holochain/holochain-rust/issues/196
pub enum ActionResponse {
    Commit(Result<Entry, HolochainError>),
    GetEntry(Option<Entry>),
    GetLinks(Result<Vec<HashString>, HolochainError>),
    LinkEntries(Result<Entry, HolochainError>),
}

impl ToJson for ActionResponse {
    fn to_json(&self) -> Result<String, HolochainError> {
        match self {
            ActionResponse::Commit(result) => match result {
                Ok(entry) => Ok(format!("{{\"hash\":\"{}\"}}", entry.key())),
                Err(err) => Ok((*err).to_json()?),
            },
            ActionResponse::GetEntry(result) => match result {
                Some(entry) => Ok(entry.to_json()?),
                None => Ok("".to_string()),
            },
            ActionResponse::GetLinks(result) => match result {
                Ok(hash_list) => Ok(json!(hash_list).to_string()),
                Err(err) => Ok((*err).to_json()?),
            },
            ActionResponse::LinkEntries(result) => match result {
                Ok(entry) => Ok(format!("{{\"hash\":\"{}\"}}", entry.key())),
                Err(err) => Ok((*err).to_json()?),
            },
        }
    }
}

/// Do a Commit Action against an agent state.
/// Intended for use inside the reducer, isolated for unit testing.
/// callback checks (e.g. validate_commit) happen elsewhere because callback functions cause
/// action reduction to hang
/// @TODO is there a way to reduce that doesn't block indefinitely on callback fns?
/// @see https://github.com/holochain/holochain-rust/issues/222
fn reduce_commit_entry(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let entry = unwrap_to!(action => Action::Commit);

    // @TODO validation dispatch should go here rather than upstream in invoke_commit
    // @see https://github.com/holochain/holochain-rust/issues/256

    let res = state.chain.push_entry(&entry);
    let response = if res.is_ok() {
        Ok(res.unwrap().entry().clone())
    } else {
        Err(res.err().unwrap())
    };

    state
        .actions
        .insert(action_wrapper.clone(), ActionResponse::Commit(response));
}

/// do a get action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn reduce_get(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let key = unwrap_to!(action => Action::GetEntry);

    let result = state.chain.entry(&key.clone());

    // @TODO if the get fails local, do a network get
    // @see https://github.com/holochain/holochain-rust/issues/167

    state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::GetEntry(result.clone()),
    );
}

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<AgentReduceFn> {
    match action_wrapper.action() {
        Action::Commit(_) => Some(reduce_commit_entry),
        Action::GetEntry(_) => Some(reduce_get),
        _ => None,
    }
}

/// Reduce Agent's state according to provided Action
pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<AgentState>,
    action_wrapper: &ActionWrapper,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) -> Arc<AgentState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: AgentState = (*old_state).clone();
            f(
                context,
                &mut new_state,
                &action_wrapper,
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
    use super::*;
    use action::tests::{test_action_wrapper_commit, test_action_wrapper_get};
    use chain::{pair::tests::test_pair, tests::test_chain};
    use error::HolochainError;
    use hash_table::entry::tests::test_entry;
    use instance::tests::{test_context, test_instance_blank};
    use std::{collections::HashMap, sync::Arc};

    /// dummy agent state
    pub fn test_agent_state() -> AgentState {
        AgentState::new(&test_chain())
    }

    /// dummy action response for a successful commit as test_pair()
    pub fn test_action_response_commit() -> ActionResponse {
        ActionResponse::Commit(Ok(test_pair().entry().clone()))
    }

    /// dummy action response for a successful get as test_pair()
    pub fn test_action_response_get() -> ActionResponse {
        ActionResponse::GetEntry(Some(test_pair().entry().clone()))
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
    /// test for the agent state actions getter
    fn agent_state_actions() {
        assert_eq!(HashMap::new(), test_agent_state().actions());
    }

    #[test]
    /// test for reducing commit entry
    fn test_reduce_commit_entry() {
        let mut state = test_agent_state();
        let action_wrapper = test_action_wrapper_commit();

        let instance = test_instance_blank();

        reduce_commit_entry(
            test_context("bob"),
            &mut state,
            &action_wrapper,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        assert_eq!(
            state.actions().get(&action_wrapper),
            Some(&test_action_response_commit()),
        );
    }

    #[test]
    fn test_reduce_get_entry() {
        let mut state = test_agent_state();
        let context = test_context("foo");

        let instance = test_instance_blank();

        let aw1 = test_action_wrapper_get();
        reduce_get(
            Arc::clone(&context),
            &mut state,
            &aw1,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        // nothing has been committed so the get must be None
        assert_eq!(
            state.actions().get(&aw1),
            Some(&ActionResponse::GetEntry(None)),
        );

        // do a round trip
        reduce_commit_entry(
            Arc::clone(&context),
            &mut state,
            &test_action_wrapper_commit(),
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        let aw2 = test_action_wrapper_get();
        reduce_get(
            Arc::clone(&context),
            &mut state,
            &aw2,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        assert_eq!(state.actions().get(&aw2), Some(&test_action_response_get()),);
    }

    #[test]
    /// test response to json
    fn test_commit_response_to_json() {
        assert_eq!(
            "{\"hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\"}",
            ActionResponse::Commit(Ok(test_pair().entry().clone()))
                .to_json()
                .unwrap(),
        );
        assert_eq!(
            "{\"error\":\"some error\"}",
            ActionResponse::Commit(Err(HolochainError::new("some error")))
                .to_json()
                .unwrap(),
        );
    }

    #[test]
    fn test_get_response_to_json() {
        assert_eq!(
            "{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}",
            ActionResponse::GetEntry(Some(test_pair().entry().clone()))
                .to_json()
                .unwrap(),
        );
        assert_eq!("", ActionResponse::GetEntry(None).to_json().unwrap());
    }

    #[test]
    fn test_get_links_response_to_json() {
        assert_eq!(
            "[\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\"]",
            ActionResponse::GetLinks(Ok(vec![test_entry().key()]))
                .to_json()
                .unwrap(),
        );
        assert_eq!(
            "{\"error\":\"some error\"}",
            ActionResponse::GetLinks(Err(HolochainError::new("some error")))
                .to_json()
                .unwrap(),
        );
    }

    #[test]
    fn test_link_entries_response_to_json() {
        assert_eq!(
            "{\"hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\"}",
            ActionResponse::LinkEntries(Ok(test_entry()))
                .to_json()
                .unwrap(),
        );
        assert_eq!(
            "{\"error\":\"some error\"}",
            ActionResponse::LinkEntries(Err(HolochainError::new("some error")))
                .to_json()
                .unwrap(),
        );
    }
}
