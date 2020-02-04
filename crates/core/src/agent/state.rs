use crate::{
    action::{Action, ActionWrapper, AgentReduceFn},
    agent::chain_store::{ChainStore, ChainStoreIterator},
    network::entry_with_header::EntryWithHeader,
    state::State,
    NEW_RELIC_LICENSE_KEY,
};
use holochain_persistence_api::cas::content::{Address, AddressableContent, Content};

use crate::{
    content_store::{AddContent, GetContent},
    state::{ActionResponse, StateWrapper, ACTION_PRUNE_MS},
};
use bitflags::_core::time::Duration;
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
use im::HashMap;
use serde_json;
use std::{convert::TryFrom, ops::Deref, sync::Arc, time::SystemTime};

/// The state-slice for the Agent.
/// Holds the agent's source chain and keys.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentState {
    /// every action and the result of that action
    actions: HashMap<ActionWrapper, Response>,
    chain_store: ChainStore,
    top_chain_header: Option<ChainHeader>,
    initial_agent_address: Address,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
    pub fn actions(&self) -> HashMap<ActionWrapper, Response> {
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
            .map(|chain_header| chain_header.entry_address().clone())
            .or_else(|| Some(self.initial_agent_address.clone()))
            .ok_or_else(|| HolochainError::ErrorGeneric("Agent entry not found".to_string()))
    }

    pub fn get_agent(&self) -> HcResult<AgentId> {
        let agent_entry_address = self.get_agent_address()?;
        let agent_entry = self
            .chain_store()
            .get(&agent_entry_address)?
            .ok_or_else(|| HolochainError::ErrorGeneric("Agent entry not found".to_string()))?;

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

pub static AGENT_SNAPSHOT_ADDRESS: &str = "AgentState";
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
#[allow(clippy::large_enum_variant)]
pub enum AgentActionResponse {
    Commit(Result<Address, HolochainError>),
    FetchEntry(Option<Entry>),
    GetLinks(Result<Vec<Address>, HolochainError>),
    LinkEntries(Result<Entry, HolochainError>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DefaultJson)]
pub struct Response(ActionResponse<AgentActionResponse>);

impl Deref for Response {
    type Target = ActionResponse<AgentActionResponse>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<AgentActionResponse> for Response {
    fn from(r: AgentActionResponse) -> Self {
        Response(ActionResponse::new(r))
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
            .map(|chain_header| chain_header.address()),
        &agent_state
            .chain_store()
            .iter_type(&agent_state.top_chain_header, &entry.entry_type())
            .nth(0)
            .map(|chain_header| chain_header.address()),
        crud_link,
        &Iso8601::new(
            duration_since_epoch.as_secs() as i64,
            duration_since_epoch.subsec_nanos(),
        ),
    ))
}

/// Create an entry-with-header for a header.
/// Since published headers are treated as entries, the header must also
/// have its own header!
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn create_entry_with_header_for_header(
    root_state: &StateWrapper,
    chain_header: ChainHeader,
) -> Result<EntryWithHeader, HolochainError> {
    let timestamp = chain_header.timestamp().clone();
    let entry = Entry::ChainHeader(chain_header);
    // This header entry needs its own header so we can publish it.
    // This is a bit delicate:
    //   * this virtual header needs to be signed
    //   * but we need to make sure that it is deterministic, i.e. that every call of this
    //     function creates the exact same header. Otherwise we end up in a endless loop of
    //     authoring new virtual headers because they have different aspect hashes due to the
    //     timestamp and the source chain link progressing over time.
    // So we first call this function that gives a new header as if it would be added to the
    // source chain...
    let proto =
        create_new_chain_header(&entry, &root_state.agent(), &root_state, &None, &Vec::new())?;
    // ... and then overwrite all links and the timestamp with static values:
    let header = ChainHeader::new(
        proto.entry_type(),
        proto.entry_address(),
        proto.provenances(),
        &None,
        &None,
        &None,
        &timestamp,
    );
    Ok(EntryWithHeader { entry, header })
}

/// Do a Commit Action against an agent state.
/// Intended for use inside the reducer, isolated for unit testing.
/// callback checks (e.g. validate_commit) happen elsewhere because callback functions cause
/// action reduction to hang
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
        agent_state.chain_store.add(entry)?;
        agent_state.chain_store.add(&chain_header)?;
        Ok((chain_header, entry.address()))
    })
    .and_then(|(chain_header, address)| {
        agent_state.top_chain_header = Some(chain_header);
        Ok(address)
    });

    agent_state.actions.insert(
        action_wrapper.clone(),
        Response::from(AgentActionResponse::Commit(result)),
    );
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn reduce_prune(agent_state: &mut AgentState, _root_state: &State, action_wrapper: &ActionWrapper) {
    assert_eq!(action_wrapper.action(), &Action::Prune);

    agent_state
        .actions
        .iter()
        .filter_map(|(action, response)| {
            if let Ok(elapsed) = response.created_at.elapsed() {
                if elapsed > Duration::from_millis(ACTION_PRUNE_MS) {
                    return Some(action);
                }
            }
            None
        })
        .cloned()
        .collect::<Vec<ActionWrapper>>()
        .into_iter()
        .for_each(|action| {
            agent_state.actions.remove(&action);
        });
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn reduce_clear_action_response(
    agent_state: &mut AgentState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let id = unwrap_to!(action => Action::ClearActionResponse);

    agent_state.actions = agent_state
        .actions
        .iter()
        .filter(|(action, _)| action.id() == id)
        .cloned()
        .collect();
}

/// maps incoming action to the correct handler
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<AgentReduceFn> {
    match action_wrapper.action() {
        Action::ClearActionResponse(_) => Some(reduce_clear_action_response),
        Action::Commit(_) => Some(reduce_commit_entry),
        Action::Prune => Some(reduce_prune),
        _ => None,
    }
}

/// Reduce Agent's state according to provided Action
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
    use im::HashMap;
    use serde_json;
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
    pub fn test_action_response_commit() -> AgentActionResponse {
        AgentActionResponse::Commit(Ok(expected_entry_address()))
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

        let response = agent_state.actions().get(&action_wrapper).unwrap().clone();
        assert_eq!(response.response(), &test_action_response_commit(),);
    }

    #[test]
    /// test response to json
    fn test_commit_response_to_json() {
        assert_eq!(
            JsonString::from_json(&format!(
                "{{\"Commit\":{{\"Ok\":\"{}\"}}}}",
                expected_entry_address()
            )),
            JsonString::from(AgentActionResponse::Commit(Ok(expected_entry_address()))),
        );
        assert_eq!(
            JsonString::from_json("{\"Commit\":{\"Err\":{\"ErrorGeneric\":\"some error\"}}}"),
            JsonString::from(AgentActionResponse::Commit(Err(HolochainError::new(
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
            JsonString::from(AgentActionResponse::FetchEntry(Some(Entry::from(
                test_entry().clone()
            ))))
        );
        assert_eq!(
            JsonString::from_json("{\"FetchEntry\":null}"),
            JsonString::from(AgentActionResponse::FetchEntry(None)),
        )
    }

    #[test]
    fn test_get_links_response_to_json() {
        assert_eq!(
            JsonString::from_json(&format!(
                "{{\"GetLinks\":{{\"Ok\":[\"{}\"]}}}}",
                expected_entry_address()
            )),
            JsonString::from(AgentActionResponse::GetLinks(Ok(vec![
                test_entry().address()
            ]))),
        );
        assert_eq!(
            JsonString::from_json("{\"GetLinks\":{\"Err\":{\"ErrorGeneric\":\"some error\"}}}"),
            JsonString::from(AgentActionResponse::GetLinks(Err(HolochainError::new(
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
            JsonString::from(AgentActionResponse::LinkEntries(Ok(Entry::from(
                test_entry(),
            )))),
        );
        assert_eq!(
            JsonString::from_json("{\"LinkEntries\":{\"Err\":{\"ErrorGeneric\":\"some error\"}}}"),
            JsonString::from(AgentActionResponse::LinkEntries(Err(HolochainError::new(
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
            &StateWrapper::from(state),
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
