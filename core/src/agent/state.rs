use action::{Action, ActionWrapper, AgentReduceFn};
use agent::chain_store::ChainStore;
use context::Context;
use holochain_cas_implementations::cas::memory::MemoryStorage;
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    chain_header::ChainHeader,
    entry::{Entry, SerializedEntry},
    error::HolochainError,
    json::JsonString,
    keys::Keys,
    signature::Signature,
    time::Iso8601,
};
use serde_json;
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
    chain: ChainStore<MemoryStorage>,
    top_chain_header: Option<ChainHeader>,
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new(chain: ChainStore<MemoryStorage>) -> AgentState {
        AgentState {
            keys: None,
            actions: HashMap::new(),
            chain,
            top_chain_header: None,
        }
    }

    /// getter for a copy of self.keys
    pub fn keys(&self) -> Option<Keys> {
        self.keys.clone()
    }

    /// getter for a copy of self.actions
    /// uniquely maps action executions to the result of the action
    pub fn actions(&self) -> HashMap<ActionWrapper, ActionResponse> {
        self.actions.clone()
    }

    pub fn chain(&self) -> ChainStore<MemoryStorage> {
        self.chain.clone()
    }

    pub fn top_chain_header(&self) -> Option<ChainHeader> {
        self.top_chain_header.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
/// the agent's response to an action
/// stored alongside the action in AgentState::actions to provide a state history that observers
/// poll and retrieve
// @TODO abstract this to a standard trait
// @see https://github.com/holochain/holochain-rust/issues/196
pub enum ActionResponse {
    Commit(Result<Address, HolochainError>),
    GetEntry(Option<SerializedEntry>),
    GetLinks(Result<Vec<Address>, HolochainError>),
    LinkEntries(Result<SerializedEntry, HolochainError>),
}

impl From<ActionResponse> for JsonString {
    fn from(action_response: ActionResponse) -> JsonString {
        JsonString::from(
            serde_json::to_string(&action_response).expect("could not Jsonify ActionResponse"),
        )
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
    let entry = unwrap_to!(action => Action::Commit);

    // @TODO validation dispatch should go here rather than upstream in invoke_commit
    // @see https://github.com/holochain/holochain-rust/issues/256

    let chain_header = ChainHeader::new(
        &entry.entry_type(),
        &entry.address(),
        // @TODO signatures
        &Signature::from(""),
        &state
            .top_chain_header
            .clone()
            .and_then(|chain_header| Some(chain_header.address())),
        &state
            .chain()
            .iter_type(&state.top_chain_header, &entry.entry_type())
            .nth(0)
            .and_then(|chain_header| Some(chain_header.address())),
        // @TODO timestamp
        &Iso8601::from(""),
    );

    // @TODO adding the entry to the CAS should happen elsewhere.
    fn response(
        state: &mut AgentState,
        entry: &Entry,
        chain_header: &ChainHeader,
    ) -> Result<Address, HolochainError> {
        state.chain.content_storage().add(entry)?;
        state.chain.content_storage().add(chain_header)?;
        Ok(entry.address())
    }
    let res = response(state, &entry, &chain_header);
    state.top_chain_header = Some(chain_header);

    state
        .actions
        .insert(action_wrapper.clone(), ActionResponse::Commit(res));
}

/// do a get action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn reduce_get_entry(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let address = unwrap_to!(action => Action::GetEntry);

    let result: Option<Entry> = state
        .chain
        .content_storage()
        .fetch(&address)
        .expect("could not fetch from CAS");

    // @TODO if the get fails local, do a network get
    // @see https://github.com/holochain/holochain-rust/issues/167

    state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::GetEntry(result.clone().and_then(|entry| Some(entry.into()))),
    );
}

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<AgentReduceFn> {
    match action_wrapper.action() {
        Action::Commit(_) => Some(reduce_commit_entry),
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
    use agent::chain_store::tests::test_chain_store;
    use holochain_core_types::{
        cas::content::AddressableContent,
        entry::{expected_entry_address, test_entry, SerializedEntry},
        error::HolochainError,
        json::{JsonString, RawString},
    };
    use instance::tests::test_context;
    use std::{collections::HashMap, sync::Arc};

    /// dummy agent state
    pub fn test_agent_state() -> AgentState {
        AgentState::new(test_chain_store())
    }

    /// dummy action response for a successful commit as test_entry()
    pub fn test_action_response_commit() -> ActionResponse {
        ActionResponse::Commit(Ok(expected_entry_address()))
    }

    /// dummy action response for a successful get as test_entry()
    pub fn test_action_response_get() -> ActionResponse {
        ActionResponse::GetEntry(Some(SerializedEntry::from(test_entry())))
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
            JsonString::from(format!(
                "{{\"Commit\":{{\"Ok\":\"{}\"}}}}",
                expected_entry_address()
            )),
            JsonString::from(ActionResponse::Commit(Ok(expected_entry_address()))),
        );
        assert_eq!(
            JsonString::from("{\"Commit\":{\"Err\":{\"ErrorGeneric\":\"some error\"}}}"),
            JsonString::from(ActionResponse::Commit(Err(HolochainError::new(
                "some error"
            ))))
        );
    }

    #[test]
    fn test_get_response_to_json() {
        assert_eq!(
            JsonString::from(
                "{\"value\":\"test entry value\",\"entry_type\":{\"App\":\"testEntryType\"}}"
            ),
            JsonString::from(ActionResponse::GetEntry(Some(SerializedEntry::from(
                test_entry().clone()
            ))))
        );
        assert_eq!(
            JsonString::from(RawString::from("")),
            JsonString::from(ActionResponse::GetEntry(None)),
        )
    }

    #[test]
    fn test_get_links_response_to_json() {
        assert_eq!(
            JsonString::from(format!(
                "{{\"GetLinks\":{{\"Ok\":[\"{}\"]}}}}",
                expected_entry_address()
            )),
            JsonString::from(ActionResponse::GetLinks(Ok(vec![test_entry().address()]))),
        );
        assert_eq!(
            JsonString::from("{\"GetLinks\":{\"Err\":{\"ErrorGeneric\":\"some error\"}}}"),
            JsonString::from(ActionResponse::GetLinks(Err(HolochainError::new(
                "some error"
            )))),
        );
    }

    #[test]
    fn test_link_entries_response_to_json() {
        assert_eq!(
            JsonString::from(format!("{{\"ok\":\"{}\"}}", expected_entry_address())),
            JsonString::from(ActionResponse::LinkEntries(Ok(SerializedEntry::from(
                test_entry(),
            )))),
        );
        assert_eq!(
            JsonString::from("{\"error\":\"some error\"}"),
            JsonString::from(ActionResponse::LinkEntries(Err(HolochainError::new(
                "some error"
            )))),
        );
    }
}
