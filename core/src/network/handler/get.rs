use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    nucleus,
};
use holochain_core_types::{cas::content::Address, eav::Attribute,entry::EntryWithMetaAndHeader};
use holochain_net::connection::json_protocol::{
    FetchEntryData, FetchEntryResultData, FetchMetaData, FetchMetaResultData,
};
use std::{collections::BTreeSet, convert::TryInto, sync::Arc};

/// The network has requested a DHT entry from us.
/// Lets try to get it and trigger a response.
pub fn handle_fetch_entry(get_dht_data: FetchEntryData, context: Arc<Context>) {
    let address = Address::from(get_dht_data.entry_address.clone());
    println!("handle fetch entry");
    let maybe_entry_with_meta =
        nucleus::actions::get_entry::get_entry_with_meta(&context, address.clone()).unwrap_or_else(
            |error| {
                context.log(format!("err/net: Error trying to find entry {:?}", error));
                None
            },
        );
    if let None = maybe_entry_with_meta 
    {
        println!("could not find entry");
        let action_wrapper = ActionWrapper::new(Action::RespondFetch((get_dht_data.clone(), None)));
        dispatch_action(context.action_channel(), action_wrapper.clone());
        return;
    };
    if let None = context.state() {
        println!("Could no get state");
        context.log(format!("err/net: Error trying to get state"));
    };
    let state = context
        .state()
        .expect("Couuld not get state for handle_fetch_entry");
    let header_res = state.get_headers(address);
    if header_res.clone().is_err() {
        println!("problem getting header");
        context.log(format!("err/net: Error trying to get headers {:?}", header_res.clone().err()));
        let action_wrapper = ActionWrapper::new(Action::RespondFetch((get_dht_data.clone(), None)));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    }
    else 
    {
        let entry_with_meta_and_header = maybe_entry_with_meta
        .map(|s| {
            Some(EntryWithMetaAndHeader{
                entry_with_meta : s.clone(),
                headers : header_res.expect("Could not get headers")
            })
        })
        .unwrap_or(None);

    let action_wrapper = ActionWrapper::new(Action::RespondFetch((get_dht_data, entry_with_meta_and_header)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    }
    
}

/// The network comes back with a result to our previous GET request.
pub fn handle_fetch_entry_result(dht_data: FetchEntryResultData, context: Arc<Context>) {
    let action_wrapper = ActionWrapper::new(Action::HandleFetchResult(dht_data));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}

pub fn handle_fetch_meta(fetch_meta_data: FetchMetaData, context: Arc<Context>) {
    if let Ok(Attribute::LinkTag(tag)) = fetch_meta_data.attribute.as_str().try_into() {
        let links = context
            .state()
            .unwrap()
            .dht()
            .get_links(
                Address::from(fetch_meta_data.entry_address.clone()),
                tag.clone(),
            )
            .unwrap_or(BTreeSet::new())
            .into_iter()
            .map(|eav| eav.value())
            .collect::<Vec<_>>();
        let action_wrapper = ActionWrapper::new(Action::RespondGetLinks((fetch_meta_data, links)));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    }
}

/// The network comes back with a result to our previous GET META request.
pub fn handle_fetch_meta_result(dht_meta_data: FetchMetaResultData, context: Arc<Context>) {
    if let Ok(Attribute::LinkTag(tag)) = dht_meta_data.attribute.as_str().try_into() {
        let action_wrapper = ActionWrapper::new(Action::HandleGetLinksResult((dht_meta_data, tag)));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    }
}
