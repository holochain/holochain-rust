use crate::{
    action::ActionWrapper,
    context::Context,
    network::{actions::ActionResponse, reducers::send, state::NetworkState},
};
use holochain_core_types::{cas::content::Address, error::HolochainError};
use holochain_net_connection::protocol_wrapper::{DhtMetaData, GetDhtMetaData, ProtocolWrapper};
use std::sync::Arc;

fn reduce_respond_get_links_inner(
    network_state: &mut NetworkState,
    get_dht_meta_data: &GetDhtMetaData,
    links: &Vec<Address>,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    send(
        network_state,
        ProtocolWrapper::GetDhtMetaResult(DhtMetaData {
            msg_id: get_dht_meta_data.msg_id.clone(),
            dna_address: network_state.dna_address.clone().unwrap(),
            agent_id: get_dht_meta_data.from_agent_id.clone(),
            from_agent_id: network_state.agent_id.clone().unwrap(),
            address: get_dht_meta_data.address.clone(),
            attribute: get_dht_meta_data.attribute.clone(),
            content: serde_json::from_str(&serde_json::to_string(&links).unwrap()).unwrap(),
        }),
    )
}

pub fn reduce_respond_get_links(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (get_dht_meta_data, links) = unwrap_to!(action => crate::action::Action::RespondGetLinks);
    let result = reduce_respond_get_links_inner(network_state, get_dht_meta_data, links);
    network_state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::RespondGetLinks(match result {
            Ok(_) => Ok(()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}
