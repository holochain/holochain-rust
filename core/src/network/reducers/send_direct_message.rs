use crate::{
    action::{ActionWrapper, DirectMessageData},
    context::Context,
    network::{reducers::send, state::NetworkState},
};
use holochain_core_types::error::HolochainError;
use holochain_net_connection::protocol_wrapper::{MessageData, ProtocolWrapper};
use std::sync::Arc;

fn inner(
    network_state: &mut NetworkState,
    direct_message_data: &DirectMessageData,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    let data = MessageData {
        msg_id: direct_message_data.msg_id.clone(),
        dna_hash: network_state.dna_hash.clone().unwrap(),
        to_agent_id: direct_message_data.address.to_string(),
        from_agent_id: network_state.agent_id.clone().unwrap(),
        data: serde_json::from_str(&serde_json::to_string(&direct_message_data.message).unwrap())
            .unwrap(),
    };

    let protocol_object = if direct_message_data.is_response {
        ProtocolWrapper::HandleSendResult(data)
    } else {
        ProtocolWrapper::SendMessage(data)
    };

    send(network_state, protocol_object)
}

pub fn reduce_send_direct_message(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let dm_data = unwrap_to!(action => crate::action::Action::SendDirectMessage);
    if let Err(error) = inner(network_state, dm_data) {
        println!("Error sending direct message: {:?}", error);
    }
}
