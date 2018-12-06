use boolinator::*;
use crate::{
    action::ActionWrapper,
    context::Context,
    network::{actions::ActionResponse, state::NetworkState},
};
use holochain_core_types::{entry::EntryWithMeta, error::HolochainError};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{DhtData, GetDhtData, ProtocolWrapper},
};
use std::sync::Arc;

fn reduce_respond_get_inner(
    network_state: &mut NetworkState,
    get_dht_data: &GetDhtData,
    maybe_entry: &Option<EntryWithMeta>,
) -> Result<(), HolochainError> {
    (network_state.network.is_some()
        && network_state.dna_hash.is_some() & network_state.agent_id.is_some())
    .ok_or("Network not initialized".to_string())?;

    let data = DhtData {
        msg_id: get_dht_data.msg_id.clone(),
        dna_hash: network_state.dna_hash.clone().unwrap(),
        agent_id: get_dht_data.from_agent_id.clone(),
        address: get_dht_data.address.clone(),
        content: serde_json::from_str(&serde_json::to_string(&maybe_entry).unwrap()).unwrap(),
    };

    network_state
        .network
        .as_mut()
        .map(|network| {
            network
                .lock()
                .unwrap()
                .send(ProtocolWrapper::GetDhtResult(data).into())
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .expect("Network has to be Some because of check above")
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
