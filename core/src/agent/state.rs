use action::{Action, ActionWrapper, AgentReduceFn};
use agent::chain_store::ChainStore;
use context::Context;
use holochain_cas_implementations::cas::file::FilesystemStorage;
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    chain_header::ChainHeader,
    entry::Entry,
    error::HolochainError,
    json::ToJson,
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
    chain: ChainStore<FilesystemStorage>,
    top_chain_header: Option<ChainHeader>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentStateSnapshot {
    top_chain_header: ChainHeader,
}

impl AgentStateSnapshot {
    pub fn top_chain_header(&self) -> &ChainHeader {
        &self.top_chain_header
    }
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new(chain: ChainStore<FilesystemStorage>) -> AgentState {
        AgentState {
            keys: None,
            actions: HashMap::new(),
            chain,
            top_chain_header: None,
        }
    }

    pub fn new_with_top_chain_header(
        chain: ChainStore<FilesystemStorage>,
        chain_header: ChainHeader,
    ) -> AgentState {
        AgentState {
            keys: None,
            actions: HashMap::new(),
            chain,
            top_chain_header: Some(chain_header),
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

    pub fn chain(&self) -> ChainStore<FilesystemStorage> {
        self.chain.clone()
    }

    pub fn top_chain_header(&self) -> Option<ChainHeader> {
        self.top_chain_header.clone()
    }
}

impl AgentStateSnapshot {
    pub fn new(chain_header: ChainHeader) -> AgentStateSnapshot {
        AgentStateSnapshot {
            top_chain_header: chain_header,
        }
    }
    pub fn from_json_str(header_str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(header_str)
    }
}

impl ToJson for AgentStateSnapshot {
    fn to_json(&self) -> Result<String, HolochainError> {
        Ok(serde_json::to_string(self)?)
    }
}

impl AddressableContent for AgentStateSnapshot {
    fn content(&self) -> Content {
        self.to_json()
            .expect("could not Jsonify ChainHeader as Content")
    }

    fn from_content(content: &Content) -> Self {
        AgentStateSnapshot::from_json_str(content)
            .expect("could not read Json as valid ChainHeader Content")
    }

    fn address(&self) -> Address {
        Address::from("AgentState")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

pub fn create_new_chain_header(entry: &Entry, agent_state: &AgentState) -> ChainHeader {
    ChainHeader::new(
        &entry.entry_type(),
        &entry.address(),
        // @TODO signatures
        &Signature::from(""),
        &agent_state
            .top_chain_header
            .clone()
            .and_then(|chain_header| Some(chain_header.address())),
        &agent_state
            .chain()
            .iter_type(&agent_state.top_chain_header, &entry.entry_type())
            .nth(0)
            .and_then(|chain_header| Some(chain_header.address())),
        // @TODO timestamp
        &Iso8601::from(""),
    )
}

/// Do a Commit Action against an agent state.
/// Intended for use inside the reducer, isolated for unit testing.
/// callback checks (e.g. validate_commit) happen elsewhere because callback functions cause
/// action reduction to hang
/// @TODO is there a way to reduce that doesn't block indefinitely on callback fns?
/// @see https://github.com/holochain/holochain-rust/issues/222
/// @TODO Better error handling in the state persister section
/// https://github.com/holochain/holochain-rust/issues/555
fn reduce_commit_entry(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let entry = unwrap_to!(action => Action::Commit);
    let chain_header = create_new_chain_header(&entry, state);

    fn response(
        state: &mut AgentState,
        entry: &Entry,
        chain_header: &ChainHeader,
    ) -> Result<Address, HolochainError> {
        state.chain.content_storage().add(entry)?;
        state.chain.content_storage().add(chain_header)?;
        Ok(entry.address())
    }
    let result = response(state, &entry, &chain_header);
    state.top_chain_header = Some(chain_header);
    let con = _context.clone();

    #[allow(unused_must_use)]
    con.state().map(|global_state_lock| {
        let persis_lock = _context.clone().persister.clone();
        let persister = &mut *persis_lock.lock().unwrap();
        persister.save(global_state_lock.clone());
    });

    state
        .actions
        .insert(action_wrapper.clone(), ActionResponse::Commit(result));
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
    let result = state
        .chain()
        .content_storage()
        .fetch(&address)
        .expect("could not fetch from CAS");
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
    extern crate tempfile;
    use self::tempfile::tempdir;
    use super::{
        reduce_commit_entry, reduce_get_entry, ActionResponse, AgentState, AgentStateSnapshot,
    };
    use action::tests::{test_action_wrapper_commit, test_action_wrapper_get};
    use agent::chain_store::{tests::test_chain_store, ChainStore};
    use holochain_cas_implementations::cas::file::FilesystemStorage;
    use holochain_core_types::{
        cas::content::AddressableContent,
        chain_header::test_chain_header,
        entry::{test_entry, test_entry_address},
        error::HolochainError,
        json::ToJson,
    };
    use instance::tests::test_context;
    use serde_json;
    use std::{collections::HashMap, sync::Arc};

    /// dummy agent state
    pub fn test_agent_state() -> AgentState {
        AgentState::new(test_chain_store())
    }

    /// dummy action response for a successful commit as test_entry()
    pub fn test_action_response_commit() -> ActionResponse {
        ActionResponse::Commit(Ok(test_entry_address()))
    }

    /// dummy action response for a successful get as test_entry()
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
            format!("{{\"address\":\"{}\"}}", test_entry_address()),
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
            "{\"value\":\"test entry value\",\"entry_type\":{\"App\":\"testEntryType\"}}",
            ActionResponse::GetEntry(Some(test_entry().clone()))
                .to_json()
                .unwrap(),
        );
        assert_eq!("", ActionResponse::GetEntry(None).to_json().unwrap());
    }

    #[test]
    fn test_get_links_response_to_json() {
        assert_eq!(
            format!("[\"{}\"]", test_entry_address()),
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
    pub fn serialize_round_trip_agent_state() {
        let header = test_chain_header();
        let agent_snap = AgentStateSnapshot::new(header);
        let json = serde_json::to_string(&agent_snap).unwrap();
        let agent_from_json: AgentStateSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(agent_snap.address(), agent_from_json.address());
    }

    #[test]
    fn test_link_entries_response_to_json() {
        assert_eq!(
            format!("{{\"address\":\"{}\"}}", test_entry_address()),
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
