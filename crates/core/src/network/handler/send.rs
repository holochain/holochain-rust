use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::direct_message::DirectMessage,
    workflows::{
        handle_custom_direct_message::handle_custom_direct_message,
        respond_validation_package_request::respond_validation_package_request,
    },
};
use std::sync::Arc;

use holochain_json_api::{error::JsonError, json::JsonString};
use lib3h_protocol::data_types::DirectMessageData;
use std::convert::TryFrom;

#[autotrace]
//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn parse_direct_message(content: &[u8]) -> Result<DirectMessage, JsonError> {
    DirectMessage::try_from(JsonString::from_json(
        std::str::from_utf8(content)
            .map_err(|error| JsonError::SerializationError(error.to_string()))?,
    ))
}

/// We got a ProtocolWrapper::SendMessage, this means somebody initiates message roundtrip
/// -> we are being called
#[autotrace]
//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn handle_send_message(message_data: DirectMessageData, context: Arc<Context>) {
    let message = match parse_direct_message(&*message_data.content.clone()) {
        Ok(message) => message,
        Err(error) => {
            log_error!(
                context,
                "net/handle_send_message: Could not deserialize DirectMessage: {:?}",
                error,
            );
            return;
        }
    };

    match message {
        DirectMessage::Custom(custom_direct_message) => {
            let c = context.clone();
            let span =
                ht::top_follower("into closure for handle_custom_direct_message".to_string());
            let closure = async move || {
                if let Err(error) = handle_custom_direct_message(
                    message_data.from_agent_id.into(),
                    message_data.request_id,
                    custom_direct_message,
                    c.clone(),
                    span,
                )
                .await
                {
                    log_error!(c, "net: Error handling custom direct message: {:?}", error);
                }
            };
            let future = closure();
            context.spawn_task(future);
        }
        DirectMessage::RequestValidationPackage(address) => {
            context.spawn_task({
                let context = context.clone();
                async move || {
                    respond_validation_package_request(
                        message_data.from_agent_id.into(),
                        message_data.request_id,
                        address,
                        context,
                        vec![],
                    );
                }
            }());
        }
        DirectMessage::ValidationPackage(_) => {
            log_error!(context,
            "net: Got DirectMessage::ValidationPackage as initial message. This should not happen.",
        )
        }
    };
}

/// We got a Lib3hClientProtocol::HandleSendMessageResult.
/// This means somebody has responded to our message that we called and this is the answer
#[autotrace]
//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn handle_send_message_result(message_data: DirectMessageData, context: Arc<Context>) {
    let response = match parse_direct_message(&message_data.content.clone()) {
        Ok(message) => message,
        Err(error) => {
            log_error!(
                context,
                "net/handle_send_message_result: Could not deserialize DirectMessage: {:?}",
                error,
            );
            return;
        }
    };

    let initial_message = context
        .network_state()
        .expect("network state not initialized")
        .direct_message_connections
        .get(&message_data.request_id)
        .cloned();

    match response {
        DirectMessage::Custom(custom_direct_message) => {
            if initial_message.is_none() {
                log_error!(context, "net: Received a custom direct message response but could not find message ID in history. Not able to process.");
                return;
            }

            let action_wrapper = ActionWrapper::new(Action::HandleCustomSendResponse((
                message_data.request_id.clone(),
                custom_direct_message.payload,
            )));
            dispatch_action(context.action_channel(), action_wrapper);

            let action_wrapper =
                ActionWrapper::new(Action::ResolveDirectConnection(message_data.request_id));
            dispatch_action(context.action_channel(), action_wrapper);
        }
        DirectMessage::RequestValidationPackage(_) => log_error!(context,
            "net: Got DirectMessage::RequestValidationPackage as a response. This should not happen.",
        ),
        DirectMessage::ValidationPackage(maybe_validation_package) => {
            if initial_message.is_none() {
                log_error!(context, "net: Received a validation package but could not find message ID in history. Not able to process.");
                return;
            }

            let initial_message = initial_message.unwrap();
            let address = unwrap_to!(initial_message => DirectMessage::RequestValidationPackage);

            let action_wrapper = ActionWrapper::new(Action::HandleGetValidationPackage((
                address.clone(),
                maybe_validation_package,
            )));
            dispatch_action(context.action_channel(), action_wrapper);

            let action_wrapper =
                ActionWrapper::new(Action::ResolveDirectConnection(message_data.request_id));
            dispatch_action(context.action_channel(), action_wrapper);
        }
    };
}
