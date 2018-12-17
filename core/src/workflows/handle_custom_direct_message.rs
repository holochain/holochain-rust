use crate::{
    action::{Action, ActionWrapper, DirectMessageData},
    context::Context,
    instance::dispatch_action,
    network::direct_message::{CustomDirectMessage, DirectMessage},
    nucleus::ribosome::callback::{receive::receive, CallbackParams, CallbackResult},
};

use holochain_core_types::{cas::content::Address, error::HolochainError};
use std::sync::Arc;

pub async fn handle_custom_direct_message(
    to_agent_id: Address,
    msg_id: String,
    custom_direct_message: CustomDirectMessage,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    let zome = custom_direct_message.zome.clone();
    let payload = custom_direct_message
        .payload
        .map_err(|error| format!("Got error in initial custom direct message: {}", error))?;

    let result = receive(context.clone(), &zome, &CallbackParams::Receive(payload));
    let response = match result {
        CallbackResult::ReceiveResult(response) => Ok(response),
        _ => Err("Error calling receive callback".to_string()),
    };

    let custom_direct_message = CustomDirectMessage {
        zome,
        payload: response,
    };
    let direct_message = DirectMessage::Custom(custom_direct_message);
    let direct_message_data = DirectMessageData {
        address: to_agent_id,
        message: direct_message,
        msg_id,
        is_response: true,
    };

    let action_wrapper = ActionWrapper::new(Action::SendDirectMessage(direct_message_data));
    dispatch_action(context.action_channel(), action_wrapper);
    Ok(())
}
