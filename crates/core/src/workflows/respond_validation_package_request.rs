use crate::{
    action::{Action, ActionWrapper, DirectMessageData},
    context::Context,
    instance::dispatch_action,
    network::direct_message::DirectMessage,
    nucleus::actions::{
        build_validation_package::build_validation_package, get_entry::get_entry_from_agent_chain,
    },

};

use holochain_core_types::signature::Provenance;
use holochain_persistence_api::cas::content::Address;
use std::{sync::Arc, vec::Vec};

#[autotrace]
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn respond_validation_package_request(
    to_agent_id: Address,
    msg_id: String,
    requested_entry_address: Address,
    context: Arc<Context>,
    provenances: Vec<Provenance>,
) {
    let maybe_validation_package =
        match get_entry_from_agent_chain(&context, &requested_entry_address) {
            Ok(Some(entry)) => build_validation_package(Arc::clone(&context), &entry, &provenances).ok(),
            _ => None,
        };

    if maybe_validation_package.is_some() {
        log_debug!(
            context,
            "workflows/validation: Sending validation package of entry {} to agent {}",
            requested_entry_address,
            to_agent_id
        );
    } else {
        log_debug!(context, "workflow/validation: Got request for validation package of unknown entry {} from agent {}!",
            requested_entry_address, to_agent_id
        );
    };

    let direct_message = DirectMessage::ValidationPackage(maybe_validation_package);
    let direct_message_data = DirectMessageData {
        address: to_agent_id,
        message: direct_message,
        msg_id,
        is_response: true,
    };

    let action_wrapper = ActionWrapper::new(Action::SendDirectMessage((direct_message_data, None)));
    dispatch_action(context.action_channel(), action_wrapper);
}
