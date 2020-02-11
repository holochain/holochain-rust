use crate::{
    action::{Action, ActionWrapper, DirectMessageData},
    context::Context,
    instance::dispatch_action,
    network::direct_message::{CustomDirectMessage, DirectMessage},
    wasm_engine::callback::{receive::receive, CallbackParams, CallbackResult},
    NEW_RELIC_LICENSE_KEY,
};

use holochain_core_types::error::HolochainError;
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::receive::ReceiveParams;
use std::sync::Arc;

/// handles receiving a message from an api send call
/// call the receive call back, and sends the result back to the
/// source of the send message which is in the from_agent_id param
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn handle_custom_direct_message(
    from_agent_id: Address,
    msg_id: String,
    custom_direct_message: CustomDirectMessage,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    let zome = custom_direct_message.zome.clone();
    let payload = custom_direct_message
        .payload
        .map_err(|error| format!("Got error in initial custom direct message: {}", error))?;

    let result = receive(
        context.clone(),
        &zome,
        &CallbackParams::Receive(ReceiveParams {
            from: from_agent_id.clone(),
            payload,
        }),
    );
    let response = match result {
        CallbackResult::ReceiveResult(response) => Ok(response),
        err => Err(format!("Error calling receive callback: {:?}", err)),
    };

    let custom_direct_message = CustomDirectMessage {
        zome,
        payload: response,
    };
    let direct_message = DirectMessage::Custom(custom_direct_message);
    let direct_message_data = DirectMessageData {
        address: from_agent_id,
        message: direct_message,
        msg_id,
        is_response: true,
    };

    let action_wrapper = ActionWrapper::new(Action::SendDirectMessage((direct_message_data, None)));
    dispatch_action(context.action_channel(), action_wrapper);
    Ok(())
}
