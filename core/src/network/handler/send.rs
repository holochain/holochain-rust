use crate::{
    context::Context, network::direct_message::DirectMessage,
    workflows::respond_validation_package_request::respond_validation_package_request,
};
use holochain_core_types::cas::content::Address;
use std::sync::Arc;

use holochain_net_connection::protocol_wrapper::MessageData;

fn log<T: Into<String>>(context: &Arc<Context>, msg: T) {
    context
        .logger
        .lock()
        .unwrap()
        .log(msg.into());
}

/// We got a ProtocolWrapper::SendMessage, this means somebody initiates message roundtrip
/// -> we are being called
pub fn handle_send(message_data: MessageData, context: Arc<Context>) {
    let message: DirectMessage =
        serde_json::from_str(&serde_json::to_string(&message_data.data).unwrap()).unwrap();

    match message {
        DirectMessage::Custom(_) => log(&context, "DirectMessage::Custom not implemented"),
        DirectMessage::RequestValidationPackage(address) => {
            respond_validation_package_request(
                Address::from(message_data.from_agent_id),
                message_data.msg_id,
                address,
                context.clone(),
            );
        }
        DirectMessage::ValidationPackage(_) => log(
            &context,
            "Got DirectMessage::ValidationPackage as initial message. This should not happen."
        ),
    }
}

/// We got a ProtocolWrapper::SendResult, this means somebody has responded to our message
/// -> we called and this is the answer
pub fn handle_send_result(message_data: MessageData, context: Arc<Context>) {
    let message: DirectMessage =
        serde_json::from_str(&serde_json::to_string(&message_data.data).unwrap()).unwrap();

    match message {
        DirectMessage::Custom(_) => log(&context, "DirectMessage::Custom not implemented"),
        DirectMessage::RequestValidationPackage(_) => log(
            &context,
            "Got DirectMessage::RequestValidationPackage as a response. This should not happen."
        ),
        DirectMessage::ValidationPackage(maybe_validation_package) => {}
    }
}
