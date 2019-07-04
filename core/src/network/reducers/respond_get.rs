use crate::{
    action::ActionWrapper,
    network::{
        actions::ActionResponse, query::NetworkQueryResult, reducers::send, state::NetworkState,
    },
    state::State,
};
use holochain_core_types::{entry::EntryWithMetaAndHeader, error::HolochainError};
use holochain_json_api::json::JsonString;
use holochain_net::connection::json_protocol::{
    JsonProtocol, QueryEntryData, QueryEntryResultData,
};

/// Send back to network a HandleQueryEntryResult, no matter what.
/// Will return an empty content field if it actually doesn't have the data.
fn reduce_respond_get_inner(
    network_state: &mut NetworkState,
    query_data: &QueryEntryData,
    maybe_entry: &Option<EntryWithMetaAndHeader>,
) -> Result<(), HolochainError> {
    network_state.initialized()?;
    let query_result_json: JsonString = NetworkQueryResult::Entry(maybe_entry.clone()).into();
    send(
        network_state,
        JsonProtocol::HandleQueryEntryResult(QueryEntryResultData {
            request_id: query_data.request_id.clone(),
            requester_agent_id: query_data.requester_agent_id.clone(),
            dna_address: network_state.dna_address.clone().unwrap(),
            responder_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry_address: query_data.entry_address.clone().into(),
            query_result: query_result_json.to_string().into_bytes(),
        }),
    )
}

pub fn reduce_respond_get(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (query_data, maybe_entry) = unwrap_to!(action => crate::action::Action::RespondGet);
    let result = reduce_respond_get_inner(network_state, query_data, maybe_entry);

    network_state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::RespondGet(match result {
            Ok(_) => Ok(()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}
