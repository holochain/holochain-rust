use crate::{
    action::ActionWrapper,
    context::Context,
    network::{actions::ActionResponse, reducers::send, state::NetworkState},
};
use holochain_core_types::{cas::content::Address, error::HolochainError};
use holochain_net_connection::json_protocol::{FetchMetaData, FetchMetaResultData, JsonProtocol};
use std::sync::Arc;

/// Send back to network a HandleFetchMetaResult, no matter what.
/// Will return an empty content field if it actually doesn't have the data.
fn reduce_respond_get_links_inner(
    network_state: &mut NetworkState,
    get_dht_meta_data: &FetchMetaData,
    links: &Vec<Address>,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    send(
        network_state,
        JsonProtocol::HandleFetchMetaResult(FetchMetaResultData {
            request_id: get_dht_meta_data.request_id.clone(),
            requester_agent_id: get_dht_meta_data.requester_agent_id.clone(),
            dna_address: network_state.dna_address.clone().unwrap(),
            provider_agent_id: network_state.agent_id.clone().unwrap(),
            entry_address: get_dht_meta_data.entry_address.clone().into(),
            attribute: get_dht_meta_data.attribute.clone(),
            content_list: vec![
                serde_json::from_str(&serde_json::to_string(&links).unwrap()).unwrap(),
            ],
        }),
    )
}

pub fn reduce_respond_get_links(
    context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (get_dht_meta_data, links) = unwrap_to!(action => crate::action::Action::RespondGetLinks);
    let result = reduce_respond_get_links_inner(network_state, get_dht_meta_data, links);

    context.log(format!(
        "debug/reduce/get_links: Responding to GET LINKS request from {} with {:?}",
        get_dht_meta_data.requester_agent_id, links
    ));

    network_state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::RespondGetLinks(match result {
            Ok(_) => Ok(()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}
