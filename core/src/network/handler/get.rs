use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    nucleus,
};
use holochain_core_types::cas::content::Address;
use holochain_net_connection::protocol_wrapper::{
    DhtData, DhtMetaData, GetDhtData, GetDhtMetaData,
};
use regex::Regex;
use std::{sync::Arc,collections::BTreeMap};

lazy_static! {
    static ref LINK: Regex =
        Regex::new(r"^link__(.*)$").expect("This string literal is a valid regex");
}

/// The network has requested a DHT entry from us.
/// Lets try to get it and trigger a response.
pub fn handle_get_dht(get_dht_data: GetDhtData, context: Arc<Context>) {
    let maybe_entry_with_meta = nucleus::actions::get_entry::get_entry_with_meta(
        &context,
        Address::from(get_dht_data.address.clone()),
    )
    .unwrap_or_else(|error| {
        context.log(format!("err/net: Error trying to find entry {:?}", error));
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

pub fn handle_get_dht_meta(get_dht_meta_data: GetDhtMetaData, context: Arc<Context>) {
    if LINK.is_match(&get_dht_meta_data.attribute) {
        let tag = LINK
            .captures(&get_dht_meta_data.attribute)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string();
        let links = context
            .state()
            .unwrap()
            .dht()
            .get_links(
                Address::from(get_dht_meta_data.address.clone()),
                tag.clone(),
            )
            .unwrap_or(BTreeMap::new())
            .into_iter()
            .map(|(_, eav)| eav.value())
            .collect::<Vec<_>>();
        let action_wrapper =
            ActionWrapper::new(Action::RespondGetLinks((get_dht_meta_data, links)));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    }
}

/// The network comes back with a result to our previous GET META request.
pub fn handle_get_dht_meta_result(dht_meta_data: DhtMetaData, context: Arc<Context>) {
    if LINK.is_match(&dht_meta_data.attribute) {
        let tag = LINK
            .captures(&dht_meta_data.attribute)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string();
        let action_wrapper = ActionWrapper::new(Action::HandleGetLinksResult((dht_meta_data, tag)));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    }
}
