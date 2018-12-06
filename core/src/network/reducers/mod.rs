pub mod get_entry;
pub mod get_validation_package;
pub mod handle_get_result;
pub mod init;
pub mod publish;
pub mod respond_get;
pub mod send_direct_message;

use boolinator::*;
use crate::{
    action::{Action, ActionWrapper, NetworkReduceFn},
    context::Context,
    network::{
        direct_message::DirectMessage,
        reducers::{
            get_entry::{reduce_get_entry, reduce_get_entry_timeout},
            handle_get_result::reduce_handle_get_result,
            init::reduce_init,
            publish::reduce_publish,
            respond_get::reduce_respond_get,
        },
        state::NetworkState,
    },
};
use holochain_core_types::{
    cas::content::Address,
    error::HolochainError,
};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{MessageData, ProtocolWrapper},
};
use snowflake::ProcessUniqueId;
use std::sync::Arc;

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<NetworkReduceFn> {
    match action_wrapper.action() {
        Action::GetEntry(_) => Some(reduce_get_entry),
        Action::GetEntryTimeout(_) => Some(reduce_get_entry_timeout),
        Action::HandleGetResult(_) => Some(reduce_handle_get_result),
        Action::InitNetwork(_) => Some(reduce_init),
        Action::Publish(_) => Some(reduce_publish),
        Action::RespondGet(_) => Some(reduce_respond_get),
        _ => None,
    }
}

pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<NetworkState>,
    action_wrapper: &ActionWrapper,
) -> Arc<NetworkState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: NetworkState = (*old_state).clone();
            f(context, &mut new_state, &action_wrapper);
            Arc::new(new_state)
        }
        None => old_state,
    }
}

pub fn initialized(network_state: &NetworkState) -> Result<(), HolochainError> {
    (network_state.network.is_some()
        && network_state.dna_hash.is_some() & network_state.agent_id.is_some())
        .ok_or(HolochainError::ErrorGeneric("Network not initialized".to_string()))
}

pub fn send(network_state: &mut NetworkState, protocol_wrapper: ProtocolWrapper) -> Result<(), HolochainError> {
    network_state
        .network
        .as_mut()
        .map(|network| {
            network
                .lock()
                .unwrap()
                .send(protocol_wrapper.into())
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .ok_or(HolochainError::ErrorGeneric("Network has to be Some because of check above".to_string()))?
}

pub fn send_message(network_state: &mut NetworkState, to_agent_id: &Address, message: DirectMessage) -> Result<(), HolochainError> {
    let id = ProcessUniqueId::new();

    let data = MessageData {
        msg_id: id.to_string(),
        dna_hash: network_state.dna_hash.clone().unwrap(),
        to_agent_id: to_agent_id.to_string(),
        from_agent_id: network_state.agent_id.clone().unwrap(),
        data: serde_json::from_str(&serde_json::to_string(&message).unwrap()).unwrap(),
    };

    let _ = send(network_state, ProtocolWrapper::SendMessage(data))?;

    network_state
        .direct_message_connections
        .insert(id, message);

    Ok(())
}