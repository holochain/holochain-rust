use crate::{
    action::ActionWrapper,
    context::Context,
    network::{actions::ActionResponse, reducers::send, state::NetworkState},
};
use holochain_core_types::{entry::EntryWithMeta, error::HolochainError};
use holochain_net_connection::protocol_wrapper::{DhtData, GetDhtData, JsonProtocol};
use std::sync::Arc;

fn reduce_respond_get_inner(
    network_state: &mut NetworkState,
    get_dht_data: &GetDhtData,
    maybe_entry: &Option<EntryWithMeta>,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    send(
        network_state,
        JsonProtocol::HandleGetDhtDataResult(DhtData {
            msg_id: get_dht_data.msg_id.clone(),
            dna_address: network_state.dna_address.clone().unwrap(),
            agent_id: get_dht_data.from_agent_id.clone(),
            address: get_dht_data.address.clone(),
            content: serde_json::from_str(&serde_json::to_string(&maybe_entry).unwrap()).unwrap(),
        }),
    )
}

pub fn reduce_respond_get(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (get_dht_data, maybe_entry) = unwrap_to!(action => crate::action::Action::RespondGet);
    let result = reduce_respond_get_inner(network_state, get_dht_data, maybe_entry);
    network_state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::RespondGet(match result {
            Ok(_) => Ok(()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}
