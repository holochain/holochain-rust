use crate::{
    action::ActionWrapper,
    network::{actions::ActionResponse, reducers::send, state::NetworkState},
    state::State,
};
use holochain_core_types::{entry::EntryWithMetaAndHeader, error::HolochainError};
use holochain_net::connection::json_protocol::{
    FetchEntryData, FetchEntryResultData, JsonProtocol,
};


//CLEANUP need to convert the param to Vec<EntryAspect> instead of maybe_entry
/// Send back to network a HandleFetchEntryResult, no matter what.
/// Will return an empty content field if it actually doesn't have the data.
fn reduce_respond_fetch_data_inner(
    network_state: &mut NetworkState,
    fetch_data: &FetchEntryData,
    maybe_entry: &Option<EntryWithMetaAndHeader>,
) -> Result<(), HolochainError> {
    network_state.initialized()?;
    let aspect_list = match maybe_entry {
        Some(entry) => vec![EntryAspcetData::from(entry)],
        None => vec![],
    };
    send(
        network_state,
        JsonProtocol::HandleFetchEntryResult(FetchEntryResultData {
            request_id: fetch_data.request_id.clone(),
            dna_address: network_state.dna_address.clone().unwrap(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: fetch_data.entry_address.clone(),
                aspect_list
            }
        }),
    )
}

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
        ActionResponse::RespondFetch(match result {
            Ok(_) => Ok(()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}
