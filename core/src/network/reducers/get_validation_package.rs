use boolinator::*;
use crate::{
    action::ActionWrapper, context::Context,
    network::{
        direct_message::DirectMessage,
        state::NetworkState
    },
};
use holochain_core_types::{
    chain_header::ChainHeader,
    error::HolochainError,
};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{MessageData, ProtocolWrapper},
};
use std::sync::Arc;

fn inner(network_state: &mut NetworkState, header: &ChainHeader) -> Result<(), HolochainError> {
    (network_state.network.is_some()
        && network_state.dna_hash.is_some() & network_state.agent_id.is_some())
        .ok_or("Network not initialized".to_string())?;

    let source_address = header.sources.first();
    let direct_message = DirectMessage::RequestValidationPackage(header.entry_address().clone());

    let data = MessageData {
        msg_id: "".to_string(),
        dna_hash: network_state.dna_hash.clone().unwrap(),
        to_agent_id: source_address,
        from_agent_id: network_state.agent_id.clone().unwrap(),
        data: serde_json::from_str(&serde_json::to_string(&direct_message).unwrap()).unwrap(),
    };

    network_state
        .network
        .as_mut()
        .map(|network| {
            network
                .lock()
                .unwrap()
                .send(ProtocolWrapper::SendMessage(data).into())
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .expect("Network has to be Some because of check above")
}

pub fn reduce_get_validation_package(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let header = unwrap_to!(action => crate::action::Action::GetValidationPackage);
    let entry_address = header.entry_address().clone();

    let result = match inner(network_state, header) {
        Ok(()) => None,
        Err(err) => Some(Err(err)),
    };

    network_state
        .get_validation_package_results
        .insert(entry_address, result);
}
