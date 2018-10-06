use action::{Action, ActionWrapper, AgentReduceFn};
use agent::keys::Keys;
use cas::content::{Address, AddressableContent};
use chain::{Chain, SourceChain};
use context::Context;
use error::HolochainError;
use hash_table::entry::Entry;
use json::ToJson;
use std::{collections::HashMap, sync::Arc};

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
    Commit(Result<Address, HolochainError>),
    GetEntry(Option<Entry>),
    GetLinks(Result<Vec<Address>, HolochainError>),
    LinkEntries(Result<Entry, HolochainError>),
}

impl ToJson for ActionResponse {
    fn to_json(&self) -> Result<String, HolochainError> {
        match self {
            ActionResponse::Commit(result) => match result {
                Ok(entry_address) => Ok(format!("{{\"address\":\"{}\"}}", entry_address)),
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
                Ok(entry) => Ok(format!("{{\"address\":\"{}\"}}", entry.address())),
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
) {
    let action = action_wrapper.action();
    let (entry_type, entry) = match action {
        Action::Commit(entry_type, entry) => (entry_type, entry),
        _ => unreachable!(),
    };

    // @TODO validation dispatch should go here rather than upstream in invoke_commit
    // @see https://github.com/holochain/holochain-rust/issues/256

    let res = state.chain.push_entry(&entry_type, &entry);
    let response = match res {
        Ok(chain_header) => Ok(chain_header.entry_address().clone()),
        Err(e) => Err(e),
    };

    state
        .actions
        .insert(action_wrapper.clone(), ActionResponse::Commit(response));
}

/// do a get action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn reduce_get_entry(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
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
        Action::Commit(_, _) => Some(reduce_commit_entry),
        Action::GetEntry(_) => Some(reduce_get_entry),
        _ => None,
    }
}

/// Reduce Agent's state according to provided Action
pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<AgentState>,
    action_wrapper: &ActionWrapper,
) -> Arc<AgentState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: AgentState = (*old_state).clone();
            f(context, &mut new_state, &action_wrapper);
            Arc::new(new_state)
        }
        None => old_state,
    }
}

#[cfg(test)]
pub mod tests {
    use super::{reduce_commit_entry, reduce_get_entry, ActionResponse, AgentState};
    use action::tests::{test_action_wrapper_commit, test_action_wrapper_get};
    use cas::content::AddressableContent;
    use chain::{pair::tests::test_pair, tests::test_chain};
    use error::HolochainError;
    use hash_table::entry::tests::{test_entry, test_entry_address};
    use instance::tests::test_context;
    use json::ToJson;
    use std::{collections::HashMap, sync::Arc};

    /// dummy agent state
    pub fn test_agent_state() -> AgentState {
        AgentState::new(&test_chain())
    }

    /// dummy action response for a successful commit as test_pair()
    pub fn test_action_response_commit() -> ActionResponse {
        ActionResponse::Commit(Ok(test_entry_address()))
    }

    /// dummy action response for a successful get as test_pair()
    pub fn test_action_response_get() -> ActionResponse {
        ActionResponse::GetEntry(Some(test_entry()))
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

        reduce_commit_entry(test_context("bob"), &mut state, &action_wrapper);

        assert_eq!(
            state.actions().get(&action_wrapper),
            Some(&test_action_response_commit()),
        );
    }

    #[test]
    /// test for reducing get entry
    fn test_reduce_get_entry() {
        let mut state = test_agent_state();
        let context = test_context("foo");

        let aw1 = test_action_wrapper_get();
        reduce_get_entry(Arc::clone(&context), &mut state, &aw1);

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
        );

        let aw2 = test_action_wrapper_get();
        reduce_get_entry(Arc::clone(&context), &mut state, &aw2);

        assert_eq!(state.actions().get(&aw2), Some(&test_action_response_get()),);
    }

    #[test]
    /// test response to json
    fn test_commit_response_to_json() {
        assert_eq!(
            "{\"address\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\"}",
            ActionResponse::Commit(Ok(test_entry_address()))
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
            "\"test entry content\"",
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
            ActionResponse::GetLinks(Ok(vec![test_entry().address()]))
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
            "{\"address\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\"}",
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
