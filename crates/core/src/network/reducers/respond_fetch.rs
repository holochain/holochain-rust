use crate::{
    action::ActionWrapper,
    network::{
        actions::NetworkActionResponse,
        entry_aspect::EntryAspect,
        reducers::{publish::entry_data_to_entry_aspect_data, send},
        state::NetworkState,
    },
    state::State,
};
use holochain_core_types::error::HolochainError;

use crate::network::actions::Response;
use lib3h_protocol::{
    data_types::{EntryData, FetchEntryData, FetchEntryResultData},
    protocol_client::Lib3hClientProtocol,
};

/// Send back to network a HandleFetchEntryResult, no matter what.
/// Will return an empty content field if it actually doesn't have the data.
[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn reduce_respond_fetch_data_inner(
    network_state: &mut NetworkState,
    fetch_data: &FetchEntryData,
    aspects: &Vec<EntryAspect>,
) -> Result<(), HolochainError> {
    network_state.initialized()?;
    send(
        network_state,
        Lib3hClientProtocol::HandleFetchEntryResult(FetchEntryResultData {
            request_id: fetch_data.request_id.clone(),
            space_address: network_state.dna_address.clone().unwrap().into(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: fetch_data.entry_address.clone(),
                aspect_list: aspects
                    .iter()
                    .map(|a| entry_data_to_entry_aspect_data(a))
                    .collect(),
            },
        }),
    )
}


[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_respond_fetch_data(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (fetch_data, maybe_entry) = unwrap_to!(action => crate::action::Action::RespondFetch);
    let result = reduce_respond_fetch_data_inner(network_state, fetch_data, maybe_entry);
    network_state.actions.insert(
        action_wrapper.clone(),
        Response::from(NetworkActionResponse::Respond(match result {
            Ok(_) => Ok(()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        })),
    );
}
