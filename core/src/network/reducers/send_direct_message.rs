use crate::{
    action::ActionWrapper,
    context::Context,
    network::{actions::ActionResponse, direct_message::DirectMessage, reducers::initialized, state::NetworkState},
};
use holochain_core_types::error::HolochainError;
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{MessageData, ProtocolWrapper},
};
use std::sync::Arc;

fn inner(
    network_state: &mut NetworkState,
    to_agent_id: String,
    direct_message: &DirectMessage,
    msg_id: String,
    is_response: bool,
) -> Result<(), HolochainError> {
    initialized(network_state)?;

    let data = MessageData {
        msg_id,
        dna_hash: network_state.dna_hash.clone().unwrap(),
        to_agent_id,
        from_agent_id: network_state.agent_id.clone().unwrap(),
        data: serde_json::from_str(&serde_json::to_string(direct_message).unwrap()).unwrap(),
    };

    let protocol_object = if is_response {
        ProtocolWrapper::SendResult(data)
    } else {
        ProtocolWrapper::SendMessage(data)
    };

    network_state
        .network
        .as_mut()
        .map(|network| {
            network
                .lock()
                .unwrap()
                .send(protocol_object.into())
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .expect("Network has to be Some because of check above")
}

pub fn reduce_send_direct_message(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (to_agent_id, direct_message, msg_id, is_response) =
        unwrap_to!(action => crate::action::Action::SendDirectMessage);

    let result = inner(
        network_state,
        to_agent_id.to_string(),
        direct_message,
        msg_id.clone(),
        *is_response,
    );

    network_state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::RespondGet(match result {
            Ok(_) => Ok(()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}
