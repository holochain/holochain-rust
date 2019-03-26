use crate::{
    action::{Action, ActionWrapper, DirectMessageData},
    context::Context,
    instance::dispatch_action,
    network::direct_message::DirectMessage,
    nucleus::actions::build_validation_package::build_validation_package,
};

use holochain_core_types::{cas::content::Address, entry::Entry, error::HolochainError};
use std::{convert::TryFrom, sync::Arc};

fn get_entry(address: &Address, context: &Arc<Context>) -> Result<Entry, HolochainError> {
    let raw = context
        .state()
        .unwrap()
        .agent()
        .chain_store()
        .content_storage()
        .read()
        .unwrap()
        .fetch(address)?
        .ok_or(HolochainError::ErrorGeneric(
            "Entry not found when trying to build validation package".to_string(),
        ))?;

    Entry::try_from(raw)
}

pub async fn respond_validation_package_request(
    to_agent_id: Address,
    msg_id: String,
    requested_entry_address: Address,
    context: Arc<Context>,
) {
    let maybe_validation_package = match get_entry(&requested_entry_address, &context) {
        Ok(entry) => await!(build_validation_package(&entry, context.clone())).ok(),
        Err(_) => None,
    };

    if maybe_validation_package.is_some() {
        context.log(format!(
            "Sending validation package of entry {} to agent {}",
            requested_entry_address, to_agent_id
        ));
    } else {
        context.log(format!(
            "Got request for validation package of unknown entry {} from agent {}!",
            requested_entry_address, to_agent_id
        ));
    };

    let direct_message = DirectMessage::ValidationPackage(maybe_validation_package);
    let direct_message_data = DirectMessageData {
        address: to_agent_id,
        message: direct_message,
        msg_id,
        is_response: true,
    };

    let action_wrapper = ActionWrapper::new(Action::SendDirectMessage(direct_message_data));
    dispatch_action(context.action_channel(), action_wrapper);
}
