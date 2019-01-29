use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    nucleus,
};
use holochain_core_types::cas::content::Address;
use holochain_net_connection::json_protocol::{
    FetchEntryData, FetchEntryResultData, FetchMetaData, FetchMetaResultData,
};
use regex::Regex;
use std::{collections::BTreeSet, sync::Arc};

lazy_static! {
    static ref LINK: Regex =
        Regex::new(r"^link__(.*)$").expect("This string literal is a valid regex");
}

/// The network has requested a DHT entry from us.
/// Lets try to get it and trigger a response.
pub fn handle_fetch_entry(get_dht_data: FetchEntryData, context: Arc<Context>) {
    let maybe_entry_with_meta = nucleus::actions::get_entry::get_entry_with_meta(
        &context,
        Address::from(get_dht_data.entry_address.clone()),
    )
    .unwrap_or_else(|error| {
        context.log(format!("err/net: Error trying to find entry {:?}", error));
        None
    });

    let action_wrapper =
        ActionWrapper::new(Action::RespondFetch((get_dht_data, maybe_entry_with_meta)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}

/// The network comes back with a result to our previous GET request.
pub fn handle_fetch_entry_result(dht_data: FetchEntryResultData, context: Arc<Context>) {
    let action_wrapper = ActionWrapper::new(Action::HandleFetchResult(dht_data));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}

pub fn handle_fetch_meta(get_dht_meta_data: FetchMetaData, context: Arc<Context>) {
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
                Address::from(get_dht_meta_data.entry_address.clone()),
                tag.clone(),
            )
            .unwrap_or(BTreeSet::new())
            .into_iter()
            .map(|eav| eav.value())
            .collect::<Vec<_>>();
        let action_wrapper =
            ActionWrapper::new(Action::RespondGetLinks((get_dht_meta_data, links)));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    }
}

/// The network comes back with a result to our previous GET META request.
pub fn handle_fetch_meta_result(dht_meta_data: FetchMetaResultData, context: Arc<Context>) {
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
