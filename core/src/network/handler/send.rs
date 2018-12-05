use crate::{
    context::Context,
    network::direct_message::DirectMessage,
    workflows::respond_validation_package_request::respond_validation_package_request,
};
use holochain_core_types::cas::content::Address;
use std::sync::Arc;

use holochain_net_connection::protocol_wrapper::{
    MessageData
};

pub fn handle_send(message_data: MessageData, context: Arc<Context>) {
    let message: DirectMessage =
        serde_json::from_str(&serde_json::to_string(&message_data.data).unwrap())
            .unwrap();

    match message {
        DirectMessage::Custom(_) => unreachable!(),
        DirectMessage::RequestValidationPackage(address) => {
            respond_validation_package_request(
                    Address::from(message_data.from_agent_id),
                    message_data.msg_id,
                    address,
                    context.clone()
            );
        },
        DirectMessage::ValidationPackage(_maybe_validation_package) => {}
    }
}

pub fn handle_send_result(_message_data: MessageData, _context: Arc<Context>) {

}

