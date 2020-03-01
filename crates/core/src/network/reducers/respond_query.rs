use crate::{
    action::ActionWrapper,
    network::{
        actions::{NetworkActionResponse, Response},
        query::NetworkQueryResult,
        reducers::send,
        state::NetworkState,
    },
    state::State,
    
};
use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use lib3h_protocol::{
    data_types::{QueryEntryData, QueryEntryResultData},
    protocol_client::Lib3hClientProtocol,
};

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn reduce_respond_query_inner(
    network_state: &mut NetworkState,
    network_query_result: &NetworkQueryResult,
    query_data: &QueryEntryData,
) -> Result<(), HolochainError> {
    network_state.initialized()?;
    let query_result_json: JsonString = network_query_result.into();
    send(
        network_state,
        Lib3hClientProtocol::HandleQueryEntryResult(QueryEntryResultData {
            request_id: query_data.request_id.clone(),
            requester_agent_id: query_data.requester_agent_id.clone(),
            space_address: network_state.dna_address.clone().unwrap().into(),
            responder_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry_address: query_data.entry_address.clone(),
            query_result: query_result_json.to_string().into_bytes().into(),
        }),
    )
}
/// Send back to network a HandleQueryEntryResult, no matter what.
/// Will return an empty content field if it actually doesn't have the data.
pub fn reduce_respond_query(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (query_data, payload) = unwrap_to!(action=>crate::action::Action::RespondQuery);
    let result = reduce_respond_query_inner(network_state, payload, query_data)
        .map(|_| Ok(()))
        .unwrap_or_else(|e| Err(HolochainError::ErrorGeneric(e.to_string())));

    network_state.actions.insert(
        action_wrapper.clone(),
        Response::from(NetworkActionResponse::Respond(result)),
    );
}
