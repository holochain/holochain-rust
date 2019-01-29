use crate::{
    action::{Action, ActionWrapper, AgentReduceFn},
    agent::chain_store::ChainStore,
    context::Context,
    state::State,
    workflows::get_entry_result::get_entry_result_workflow,
};
use holochain_core_types::{
    agent::AgentId,
    cas::content::{Address, AddressableContent, Content},
    chain_header::ChainHeader,
    entry::{entry_type::EntryType, Entry},
    error::{HcResult, HolochainError},
    json::*,
    signature::Signature,
    time::Iso8601,
};
use holochain_wasm_utils::api_serialization::get_entry::*;
use serde_json;
use std::{collections::HashMap, convert::TryFrom, sync::Arc};

/// The state-slice for the Agent.
/// Holds the agent's source chain and keys.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentState {
    /// every action and the result of that action
    // @TODO this will blow up memory, implement as some kind of dropping/FIFO with a limit?
    // @see https://github.com/holochain/holochain-rust/issues/166
    actions: HashMap<ActionWrapper, ActionResponse>,
    chain: ChainStore,
    top_chain_header: Option<ChainHeader>,
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new(chain: ChainStore) -> AgentState {
        AgentState {
            actions: HashMap::new(),
            chain,
            top_chain_header: None,
        }
    }

    pub fn new_with_top_chain_header(chain: ChainStore, chain_header: ChainHeader) -> AgentState {
        AgentState {
            actions: HashMap::new(),
            chain,
            top_chain_header: Some(chain_header),
        }
    }

    /// getter for a copy of self.actions
    /// uniquely maps action executions to the result of the action
    pub fn actions(&self) -> HashMap<ActionWrapper, ActionResponse> {
        self.actions.clone()
    }

    pub fn chain(&self) -> ChainStore {
        self.chain.clone()
    }

    pub fn top_chain_header(&self) -> Option<ChainHeader> {
        self.top_chain_header.clone()
    }

    pub fn get_agent_address(&self) -> HcResult<Address> {
        self.chain()
            .iter_type(&self.top_chain_header, &EntryType::AgentId)
            .nth(0)
            .and_then(|chain_header| Some(chain_header.entry_address().clone()))
            .ok_or(HolochainError::ErrorGeneric(
                "Agent entry not found".to_string(),
            ))
    }

    pub async fn get_agent<'a>(&'a self, context: &'a Arc<Context>) -> HcResult<AgentId> {
        let agent_entry_address = self.get_agent_address()?;
        let entry_args = GetEntryArgs {
            address: agent_entry_address,
            options: Default::default(),
        };
        let agent_entry_result = await!(get_entry_result_workflow(context, &entry_args))?;
        let agent_entry = agent_entry_result.latest();
        match agent_entry {
            None => Err(HolochainError::ErrorGeneric(
                "Agent entry not found".to_string(),
            )),
            Some(Entry::AgentId(agent_id)) => Ok(agent_id),
            _ => unreachable!(),
        }
    }

    pub fn get_header_for_entry(&self, entry: &Entry) -> Option<ChainHeader> {
        self.chain()
            .iter_type(&self.top_chain_header(), &entry.entry_type())
            .find(|h| h.entry_address() == &entry.address())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, DefaultJson)]
pub struct AgentStateSnapshot {
    top_chain_header: ChainHeader,
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
    pub fn top_chain_header(&self) -> &ChainHeader {
        &self.top_chain_header
    }
}

impl TryFrom<State> for AgentStateSnapshot {
    type Error = HolochainError;

    fn try_from(state: State) -> Result<Self, Self::Error> {
        let agent = &*(state.agent());
        let top_chain = agent
            .top_chain_header()
            .ok_or_else(|| HolochainError::ErrorGeneric("Could not serialize".to_string()))?;
        Ok(AgentStateSnapshot::new(top_chain))
    }
}

pub static AGENT_SNAPSHOT_ADDRESS: &'static str = "AgentState";
impl AddressableContent for AgentStateSnapshot {
    fn content(&self) -> Content {
        self.to_owned().into()
    }

    fn try_from_content(content: &Content) -> Result<Self, HolochainError> {
        Self::try_from(content.to_owned())
    }

    fn address(&self) -> Address {
        AGENT_SNAPSHOT_ADDRESS.into()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DefaultJson)]
/// the agent's response to an action
/// stored alongside the action in AgentState::actions to provide a state history that observers
/// poll and retrieve
// @TODO abstract this to a standard trait
// @see https://github.com/holochain/holochain-rust/issues/196
pub enum ActionResponse {
    Commit(Result<Address, HolochainError>),
    FetchEntry(Option<Entry>),
    GetLinks(Result<Vec<Address>, HolochainError>),
    LinkEntries(Result<Entry, HolochainError>),
}

pub fn create_new_chain_header(
    entry: &Entry,
    context: Arc<Context>,
    crud_link: &Option<Address>,
) -> ChainHeader {
    let agent_state = context
        .state()
        .expect("create_new_chain_header called without state")
        .agent();
    let agent_address = agent_state
        .get_agent_address()
        .unwrap_or(context.agent_id.address());
    ChainHeader::new(
        &entry.entry_type(),
        &entry.address(),
        &vec![agent_address],
        // @TODO signatures
        &vec![Signature::from("")],
        &agent_state
            .top_chain_header
            .clone()
            .and_then(|chain_header| Some(chain_header.address())),
        &agent_state
            .chain()
            .iter_type(&agent_state.top_chain_header, &entry.entry_type())
            .nth(0)
            .and_then(|chain_header| Some(chain_header.address())),
        crud_link,
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
    context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (entry, maybe_crud_link) = unwrap_to!(action => Action::Commit);
    let chain_header = create_new_chain_header(&entry, context.clone(), &maybe_crud_link);

    fn response(
        state: &mut AgentState,
        entry: &Entry,
        chain_header: &ChainHeader,
    ) -> Result<Address, HolochainError> {
        let storage = &state.chain.content_storage().clone();
        storage.write().unwrap().add(entry)?;
        storage.write().unwrap().add(chain_header)?;
        Ok(entry.address())
    }
    let result = response(state, &entry, &chain_header);
    state.top_chain_header = Some(chain_header);
    let con = context.clone();

    #[allow(unused_must_use)]
    con.state().map(|global_state_lock| {
        let persis_lock = context.clone().persister.clone();
        let persister = &mut *persis_lock.lock().unwrap();
        persister.save(global_state_lock.clone());
    });

    state
        .actions
        .insert(action_wrapper.clone(), ActionResponse::Commit(result));
}

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<AgentReduceFn> {
    match action_wrapper.action() {
        Action::Commit(_) => Some(reduce_commit_entry),
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
    use super::{reduce_commit_entry, ActionResponse, AgentState, AgentStateSnapshot};
    use crate::{
        action::tests::test_action_wrapper_commit, agent::chain_store::tests::test_chain_store,
        instance::tests::test_context, state::State,
    };
    use holochain_core_types::{
        cas::content::AddressableContent,
        chain_header::test_chain_header,
        entry::{expected_entry_address, test_entry, Entry},
        error::HolochainError,
        json::JsonString,
    };
    use serde_json;
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    /// dummy agent state
    pub fn test_agent_state() -> AgentState {
        AgentState::new(test_chain_store())
    }

    /// dummy action response for a successful commit as test_entry()
    pub fn test_action_response_commit() -> ActionResponse {
        ActionResponse::Commit(Ok(expected_entry_address()))
    }

    #[test]
    /// smoke test for building a new AgentState
    fn agent_state_new() {
        test_agent_state();
    }

    #[test]
    /// test for the agent state actions getter
    fn agent_state_actions() {
        assert_eq!(HashMap::new(), test_agent_state().actions());
    }

    #[test]
    /// test for reducing commit entry
    fn test_reduce_commit_entry() {
        let mut agent_state = test_agent_state();
        let netname = Some("test_reduce_commit_entry");
        let context = test_context("bob", netname);
        let state = State::new_with_agent(context, Arc::new(agent_state.clone()));
        let mut context = test_context("bob", netname);
        Arc::get_mut(&mut context)
            .unwrap()
            .set_state(Arc::new(RwLock::new(state)));
        let action_wrapper = test_action_wrapper_commit();

        reduce_commit_entry(context, &mut agent_state, &action_wrapper);

        assert_eq!(
            agent_state.actions().get(&action_wrapper),
            Some(&test_action_response_commit()),
        );
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
                "{\"FetchEntry\":{\"App\":[\"testEntryType\",\"\\\"test entry value\\\"\"]}}"
            ),
            JsonString::from(ActionResponse::FetchEntry(Some(Entry::from(
                test_entry().clone()
            ))))
        );
        assert_eq!(
            JsonString::from("{\"FetchEntry\":null}"),
            JsonString::from(ActionResponse::FetchEntry(None)),
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
    pub fn serialize_round_trip_agent_state() {
        let header = test_chain_header();
        let agent_snap = AgentStateSnapshot::new(header);
        let json = serde_json::to_string(&agent_snap).unwrap();
        let agent_from_json = AgentStateSnapshot::from_json_str(&json).unwrap();
        assert_eq!(agent_snap.address(), agent_from_json.address());
    }

    #[test]
    fn test_link_entries_response_to_json() {
        assert_eq!(
            JsonString::from("{\"LinkEntries\":{\"Ok\":{\"App\":[\"testEntryType\",\"\\\"test entry value\\\"\"]}}}"),
            JsonString::from(ActionResponse::LinkEntries(Ok(Entry::from(
                test_entry(),
            )))),
        );
        assert_eq!(
            JsonString::from("{\"LinkEntries\":{\"Err\":{\"ErrorGeneric\":\"some error\"}}}"),
            JsonString::from(ActionResponse::LinkEntries(Err(HolochainError::new(
                "some error"
            )))),
        );
    }
}
