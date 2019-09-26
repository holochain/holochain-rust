use crate::{
    action::{Action, ActionWrapper, AgentReduceFn},
    agent::chain_store::{ChainStore, ChainStoreIterator},
    state::State,
};
use holochain_persistence_api::cas::content::{Address, AddressableContent, Content};

use crate::state::StateWrapper;
use holochain_core_types::{
    agent::AgentId,
    chain_header::ChainHeader,
    entry::{entry_type::EntryType, Entry},
    error::{HcResult, HolochainError},
    signature::{Provenance, Signature},
    time::Iso8601,
};
use holochain_json_api::{
    error::{JsonError, JsonResult},
    json::JsonString,
};
use holochain_wasm_utils::api_serialization::crypto::CryptoMethod;
use serde_json;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    sync::Arc,
    time::SystemTime,
};

/// The state-slice for the Agent.
/// Holds the agent's source chain and keys.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentState {
    /// every action and the result of that action
    // @TODO this will blow up memory, implement as some kind of dropping/FIFO with a limit?
    // @see https://github.com/holochain/holochain-rust/issues/166
    actions: HashMap<ActionWrapper, ActionResponse>,
    chain_store: ChainStore,
    top_chain_header: Option<ChainHeader>,
    initial_agent_address: Address,
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new(chain_store: ChainStore, initial_agent_address: Address) -> AgentState {
        AgentState {
            actions: HashMap::new(),
            chain_store,
            top_chain_header: None,
            initial_agent_address,
        }
    }

    pub fn new_with_top_chain_header(
        chain_store: ChainStore,
        chain_header: Option<ChainHeader>,
        initial_agent_address: Address,
    ) -> AgentState {
        AgentState {
            actions: HashMap::new(),
            chain_store,
            top_chain_header: chain_header,
            initial_agent_address,
        }
    }

    /// getter for a copy of self.actions
    /// uniquely maps action executions to the result of the action
    pub fn actions(&self) -> HashMap<ActionWrapper, ActionResponse> {
        self.actions.clone()
    }

    pub fn chain_store(&self) -> ChainStore {
        self.chain_store.clone()
    }

    pub fn top_chain_header(&self) -> Option<ChainHeader> {
        self.top_chain_header.clone()
    }

    pub fn iter_chain(&self) -> ChainStoreIterator {
        self.chain_store.iter(&self.top_chain_header)
    }

    pub fn get_agent_address(&self) -> HcResult<Address> {
        self.chain_store()
            .iter_type(&self.top_chain_header, &EntryType::AgentId)
            .nth(0)
            .and_then(|chain_header| Some(chain_header.entry_address().clone()))
            .or_else(|| Some(self.initial_agent_address.clone()))
            .ok_or_else(|| HolochainError::ErrorGeneric("Agent entry not found".to_string()))
    }

    pub fn get_agent(&self) -> HcResult<AgentId> {
        let agent_entry_address = self.get_agent_address()?;
        let maybe_agent_entry_json = self
            .chain_store()
            .content_storage()
            .read()?
            .fetch(&agent_entry_address)?;
        let agent_entry_json = maybe_agent_entry_json
            .ok_or_else(|| HolochainError::ErrorGeneric("Agent entry not found".to_string()))?;

        let agent_entry: Entry = agent_entry_json.try_into()?;
        match agent_entry {
            Entry::AgentId(agent_id) => Ok(agent_id),
            _ => unreachable!(),
        }
    }

    pub fn get_most_recent_header_for_entry(&self, entry: &Entry) -> Option<ChainHeader> {
        self.chain_store()
            .iter_type(&self.top_chain_header(), &entry.entry_type())
            .find(|h| h.entry_address() == &entry.address())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, DefaultJson)]
pub struct AgentStateSnapshot {
    top_chain_header: Option<ChainHeader>,
}

impl AgentStateSnapshot {
    pub fn new(chain_header: Option<ChainHeader>) -> AgentStateSnapshot {
        AgentStateSnapshot {
            top_chain_header: chain_header,
        }
    }
    pub fn from_json_str(header_str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(header_str)
    }
    pub fn top_chain_header(&self) -> Option<&ChainHeader> {
        self.top_chain_header.as_ref()
    }
}

impl From<&StateWrapper> for AgentStateSnapshot {
    fn from(state: &StateWrapper) -> Self {
        let agent = &*(state.agent());
        let top_chain = agent.top_chain_header();
        AgentStateSnapshot::new(top_chain)
    }
}

pub static AGENT_SNAPSHOT_ADDRESS: &'static str = "AgentState";
impl AddressableContent for AgentStateSnapshot {
    fn content(&self) -> Content {
        self.to_owned().into()
    }

    fn try_from_content(content: &Content) -> JsonResult<Self> {
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
#[allow(clippy::large_enum_variant)]
pub enum ActionResponse {
    Commit(Result<Address, HolochainError>),
    FetchEntry(Option<Entry>),
    GetLinks(Result<Vec<Address>, HolochainError>),
    LinkEntries(Result<Entry, HolochainError>),
}

pub fn create_new_chain_header(
    entry: &Entry,
    agent_state: &AgentState,
    root_state: &StateWrapper,
    crud_link: &Option<Address>,
    provenances: &Vec<Provenance>,
) -> Result<ChainHeader, HolochainError> {
    let agent_address = agent_state.get_agent_address()?;
    let signature = Signature::from(
        root_state
            .conductor_api()
            .execute(entry.address().to_string(), CryptoMethod::Sign)?,
        // Temporarily replaced by error handling for Holo hack signing.
        // TODO: pull in the expect below after removing the Holo signing hack again
        //.expect("Must be able to create signatures!"),
    );
    let duration_since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time must not be before UNIX EPOCH");

    let mut provenances: Vec<Provenance> = provenances.to_vec();
    provenances.push(Provenance::new(agent_address, signature));

    Ok(ChainHeader::new(
        &entry.entry_type(),
        &entry.address(),
        &provenances,
        &agent_state
            .top_chain_header
            .clone()
            .and_then(|chain_header| Some(chain_header.address())),
        &agent_state
            .chain_store()
            .iter_type(&agent_state.top_chain_header, &entry.entry_type())
            .nth(0)
            .and_then(|chain_header| Some(chain_header.address())),
        crud_link,
        &Iso8601::from(duration_since_epoch.as_secs()),
    ))
}

/// Do a Commit Action against an agent state.
/// Intended for use inside the reducer, isolated for unit testing.
/// callback checks (e.g. validate_commit) happen elsewhere because callback functions cause
/// action reduction to hang
fn reduce_commit_entry(
    agent_state: &mut AgentState,
    root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (entry, maybe_link_update_delete, provenances) = unwrap_to!(action => Action::Commit);

    let result = create_new_chain_header(
        &entry,
        agent_state,
        &StateWrapper::from(root_state.clone()),
        &maybe_link_update_delete,
        provenances,
    )
    .and_then(|chain_header| {
        let storage = &agent_state.chain_store.content_storage().clone();
        storage.write().unwrap().add(entry)?;
        storage.write().unwrap().add(&chain_header)?;
        Ok((chain_header, entry.address()))
    })
    .and_then(|(chain_header, address)| {
        agent_state.top_chain_header = Some(chain_header);
        Ok(address)
    });

    agent_state
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
    old_state: Arc<AgentState>,
    root_state: &State,
    action_wrapper: &ActionWrapper,
) -> Arc<AgentState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: AgentState = (*old_state).clone();
            f(&mut new_state, root_state, &action_wrapper);
            Arc::new(new_state)
        }
        None => old_state,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        action::tests::test_action_wrapper_commit, agent::chain_store::tests::test_chain_store,
        instance::tests::test_context, state::State,
    };
    use holochain_core_types::{
        chain_header::{test_chain_header, ChainHeader},
        entry::{expected_entry_address, test_entry, Entry},
        error::HolochainError,
        signature::Signature,
    };
    use holochain_json_api::json::JsonString;
    use holochain_persistence_api::cas::content::AddressableContent;
    use serde_json;
    use std::collections::HashMap;
    use test_utils::mock_signing::mock_signer;

    /// dummy agent state
    pub fn test_agent_state(maybe_initial_agent_address: Option<Address>) -> AgentState {
        AgentState::new(
            test_chain_store(),
            maybe_initial_agent_address
                .or_else(|| Some(AgentId::generate_fake("test agent").address()))
                .unwrap(),
        )
    }

    /// dummy action response for a successful commit as test_entry()
    pub fn test_action_response_commit() -> ActionResponse {
        ActionResponse::Commit(Ok(expected_entry_address()))
    }

    #[test]
    /// smoke test for building a new AgentState
    fn agent_state_new() {
        test_agent_state(None);
    }

    #[test]
    /// test for the agent state actions getter
    fn agent_state_actions() {
        assert_eq!(HashMap::new(), test_agent_state(None).actions());
    }

    #[test]
    /// test for reducing commit entry
    fn test_reduce_commit_entry() {
        let netname = Some("test_reduce_commit_entry");
        let context = test_context("bob", netname);
        let mut agent_state = test_agent_state(Some(context.agent_id.address()));
        let state = State::new_with_agent(context, agent_state.clone());
        let action_wrapper = test_action_wrapper_commit();

        reduce_commit_entry(&mut agent_state, &state, &action_wrapper);

        assert_eq!(
            agent_state.actions().get(&action_wrapper),
            Some(&test_action_response_commit()),
        );
    }

    #[test]
    /// test response to json
    fn test_commit_response_to_json() {
        assert_eq!(
            JsonString::from_json(&format!(
                "{{\"Commit\":{{\"Ok\":\"{}\"}}}}",
                expected_entry_address()
            )),
            JsonString::from(ActionResponse::Commit(Ok(expected_entry_address()))),
        );
        assert_eq!(
            JsonString::from_json("{\"Commit\":{\"Err\":{\"ErrorGeneric\":\"some error\"}}}"),
            JsonString::from(ActionResponse::Commit(Err(HolochainError::new(
                "some error"
            ))))
        );
    }

    #[test]
    fn test_get_response_to_json() {
        assert_eq!(
            JsonString::from_json(
                "{\"FetchEntry\":{\"App\":[\"testEntryType\",\"\\\"test entry value\\\"\"]}}"
            ),
            JsonString::from(ActionResponse::FetchEntry(Some(Entry::from(
                test_entry().clone()
            ))))
        );
        assert_eq!(
            JsonString::from_json("{\"FetchEntry\":null}"),
            JsonString::from(ActionResponse::FetchEntry(None)),
        )
    }

    #[test]
    fn test_get_links_response_to_json() {
        assert_eq!(
            JsonString::from_json(&format!(
                "{{\"GetLinks\":{{\"Ok\":[\"{}\"]}}}}",
                expected_entry_address()
            )),
            JsonString::from(ActionResponse::GetLinks(Ok(vec![test_entry().address()]))),
        );
        assert_eq!(
            JsonString::from_json("{\"GetLinks\":{\"Err\":{\"ErrorGeneric\":\"some error\"}}}"),
            JsonString::from(ActionResponse::GetLinks(Err(HolochainError::new(
                "some error"
            )))),
        );
    }

    #[test]
    pub fn serialize_round_trip_agent_state() {
        let header = test_chain_header();
        let agent_snap = AgentStateSnapshot::new(Some(header));
        let json = serde_json::to_string(&agent_snap).unwrap();
        let agent_from_json = AgentStateSnapshot::from_json_str(&json).unwrap();
        assert_eq!(agent_snap.address(), agent_from_json.address());
    }

    #[test]
    fn test_link_entries_response_to_json() {
        assert_eq!(
            JsonString::from_json("{\"LinkEntries\":{\"Ok\":{\"App\":[\"testEntryType\",\"\\\"test entry value\\\"\"]}}}"),
            JsonString::from(ActionResponse::LinkEntries(Ok(Entry::from(
                test_entry(),
            )))),
        );
        assert_eq!(
            JsonString::from_json("{\"LinkEntries\":{\"Err\":{\"ErrorGeneric\":\"some error\"}}}"),
            JsonString::from(ActionResponse::LinkEntries(Err(HolochainError::new(
                "some error"
            )))),
        );
    }

    #[test]
    fn test_create_new_chain_header() {
        let netname = Some("test_create_new_chain_header");
        let context = test_context("bob", netname);
        let agent_state = test_agent_state(Some(context.agent_id.address()));
        let state = State::new_with_agent(context.clone(), agent_state.clone());

        let header = create_new_chain_header(
            &test_entry(),
            &agent_state,
            &StateWrapper::from(state.clone()),
            &None,
            &vec![],
        )
        .unwrap();
        let agent_id = context.agent_id.clone();
        assert_eq!(
            header,
            ChainHeader::new(
                &test_entry().entry_type(),
                &test_entry().address(),
                &[Provenance::new(
                    agent_id.address(),
                    Signature::from(mock_signer(test_entry().address().to_string(), &agent_id))
                )]
                .to_vec(),
                &None,
                &None,
                &None,
                &header.timestamp(),
            )
        );
    }
}
