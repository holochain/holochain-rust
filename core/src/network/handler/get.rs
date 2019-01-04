use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    nucleus,
};
use holochain_core_types::cas::content::Address;
use std::sync::Arc;

use holochain_net_connection::protocol_wrapper::{DhtData, GetDhtData};

/// The network has requested a DHT entry from us.
/// Lets try to get it and trigger a response.
pub fn handle_get_dht(get_dht_data: GetDhtData, context: Arc<Context>) {
    let maybe_entry_with_meta = nucleus::actions::get_entry::get_entry_with_meta(
        &context,
        Address::from(get_dht_data.address.clone()),
    )
    .unwrap_or_else(|error| {
        context.log(format!("error/net: Error trying to find entry {:?}", error));
        None
    });

    let action_wrapper =
        ActionWrapper::new(Action::RespondGet((get_dht_data, maybe_entry_with_meta)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}

/// The network comes back with a result to our previous GET request.
pub fn handle_get_dht_result(dht_data: DhtData, context: Arc<Context>) {
    let action_wrapper = ActionWrapper::new(Action::HandleGetResult(dht_data));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}
