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
use holochain_persistence_api::cas::content::Address;
use std::{sync::Arc, thread};

use holochain_json_api::{error::JsonError, json::JsonString};
use lib3h_protocol::data_types::DirectMessageData;
use snowflake::ProcessUniqueId;
use std::convert::TryFrom;

fn parse_direct_message(content: Vec<u8>) -> Result<DirectMessage, JsonError> {
    DirectMessage::try_from(JsonString::from_json(
        &String::from_utf8(content)
            .map_err(|error| JsonError::SerializationError(error.to_string()))?,
    ))
}

/// We got a ProtocolWrapper::SendMessage, this means somebody initiates message roundtrip
/// -> we are being called
pub fn handle_send_message(message_data: DirectMessageData, context: Arc<Context>) {
    let message = match parse_direct_message(message_data.content.clone()) {
        Ok(message) => message,
        Err(error) => {
            log_error!(context,
                "net/handle_send_message: Could not deserialize DirectMessage: {:?}",
                error,
            );
            return;
        }
    };

    match message {
        DirectMessage::Custom(custom_direct_message) => {
            thread::Builder::new()
                .name(format!(
                    "custom_direct_message/{}",
                    ProcessUniqueId::new().to_string()
                ))
                .spawn(move || {
                    if let Err(error) = context.block_on(handle_custom_direct_message(
                        Address::from(message_data.from_agent_id),
                        message_data.request_id,
                        custom_direct_message,
                        context.clone(),
                    )) {
                        log_error!(context,
                            "net: Error handling custom direct message: {:?}",
                            error
                        );
                    }
                })
                .expect("Could not spawn thread for handling of custom direct message");
        }
        DirectMessage::RequestValidationPackage(address) => {
            // Async functions only get executed when they are polled.
            // I don't want to wait for this workflow to finish here as it would block the
            // network thread, so I use block_on to poll the async function but do that in
            // another thread:
            thread::Builder::new()
                .name(format!(
                    "validation_package_request/{}",
                    ProcessUniqueId::new().to_string()
                ))
                .spawn(move || {
                    context.block_on(respond_validation_package_request(
                        Address::from(message_data.from_agent_id),
                        message_data.request_id,
                        address,
                        context.clone(),
                        &vec![],
                    ));
                })
                .expect("Could not spawn thread for handling of validation package request");
        }
        DirectMessage::ValidationPackage(_) => log_error!(context, 
            "net: Got DirectMessage::ValidationPackage as initial message. This should not happen.",
        ),
    };
}

/// We got a Lib3hClientProtocol::HandleSendMessageResult.
/// This means somebody has responded to our message that we called and this is the answer
pub fn handle_send_message_result(message_data: DirectMessageData, context: Arc<Context>) {
    let response = match parse_direct_message(message_data.content.clone()) {
        Ok(message) => message,
        Err(error) => {
            log_error!(context,
                "net/handle_send_message_result: Could not deserialize DirectMessage: {:?}",
                error,
            );
            return;
        }
    };

    let initial_message = context
        .state()
        .unwrap()
        .network()
        .as_ref()
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
            dispatch_action(context.action_channel(), action_wrapper.clone());

            let action_wrapper =
                ActionWrapper::new(Action::ResolveDirectConnection(message_data.request_id));
            dispatch_action(context.action_channel(), action_wrapper.clone());
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
                maybe_validation_package.clone(),
            )));
            dispatch_action(context.action_channel(), action_wrapper.clone());

            let action_wrapper =
                ActionWrapper::new(Action::ResolveDirectConnection(message_data.request_id));
            dispatch_action(context.action_channel(), action_wrapper.clone());
        }
    };
}
