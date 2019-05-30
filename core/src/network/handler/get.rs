use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    entry::CanPublish,
    instance::dispatch_action,
    nucleus,
};
use holochain_core_types::{cas::content::Address, eav::Attribute, entry::EntryWithMetaAndHeader};
use holochain_net::connection::json_protocol::{
    FetchEntryData, FetchEntryResultData, FetchMetaData, FetchMetaResultData,
};
use std::{collections::BTreeSet, convert::TryInto, sync::Arc};

/// The network has requested a DHT entry from us.
/// Lets try to get it and trigger a response.
pub fn handle_fetch_entry(get_dht_data: FetchEntryData, context: Arc<Context>) {
    let address = Address::from(get_dht_data.entry_address.clone());
    let get_entry = nucleus::actions::get_entry::get_entry_with_meta(&context, address.clone())
        .map(|entry_with_meta_opt| {
            let state = context
                .state()
                .expect("Could not get state for handle_fetch_entry");
            state
                .get_headers(address)
                .map(|headers| {
                    entry_with_meta_opt
                        .map(|entry_with_meta| {
                            if entry_with_meta.entry.entry_type().can_publish(&context) {
                                Some(EntryWithMetaAndHeader {
                                    entry_with_meta: entry_with_meta.clone(),
                                    headers,
                                })
                            } else {
                                None
                            }
                        })
                        .unwrap_or(None)
                })
                .map_err(|error| {
                    context.log(format!("err/net: Error trying to get headers {:?}", error));
                    None::<EntryWithMetaAndHeader>
                })
        })
        .map_err(|error| {
            context.log(format!("err/net: Error trying to find entry {:?}", error));
            None::<EntryWithMetaAndHeader>
        })
        .unwrap_or(Ok(None));
    let action_wrapper = ActionWrapper::new(Action::RespondFetch((
        get_dht_data,
        get_entry.unwrap_or(None),
    )));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}

/// The network comes back with a result to our previous GET request.
pub fn handle_fetch_entry_result(dht_data: FetchEntryResultData, context: Arc<Context>) {
    let action_wrapper = ActionWrapper::new(Action::HandleFetchResult(dht_data));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}

pub fn handle_fetch_meta(fetch_meta_data: FetchMetaData, context: Arc<Context>) {
    if let Ok(Attribute::LinkTag(link_type, tag)) = fetch_meta_data.attribute.as_str().try_into() {
        let unwrapped_tag: Option<String> = if &tag == "*" { None} else { Some(tag)};
        let unwrapped_link_type: Option<String> = if &link_type == "*" { None} else { Some(link_type)};
        let links = context
            .state()
            .unwrap()
            .dht()
            .get_links(
                Address::from(fetch_meta_data.entry_address.clone()),
                unwrapped_link_type.clone(),
                unwrapped_tag.clone(),
            )
            .unwrap_or(BTreeSet::new())
            .into_iter()
            .map(|eav_crud| (eav_crud.0.value(),eav_crud.1))
            .collect::<Vec<_>>();
        let action_wrapper = ActionWrapper::new(Action::RespondGetLinks((fetch_meta_data, links)));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    }
}

/// The network comes back with a result to our previous GET META request.
pub fn handle_fetch_meta_result(dht_meta_data: FetchMetaResultData, context: Arc<Context>) {
    if let Ok(Attribute::LinkTag(link_type, tag)) = dht_meta_data.attribute.as_str().try_into() {
        let action_wrapper = ActionWrapper::new(Action::HandleGetLinksResult((
            dht_meta_data,
            link_type,
            tag,
        )));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    }
}
