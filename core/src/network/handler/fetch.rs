use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    entry::CanPublish,
    instance::dispatch_action,
    network::entry_aspect::EntryAspect,
    nucleus,
};
use holochain_core_types::cas::content::Address;
use holochain_net::connection::json_protocol::FetchEntryData;
use std::sync::Arc;
use holochain_core_types::error::HolochainError;
use boolinator::*;

/// The network has requested a DHT entry from us.
/// Lets try to get it and trigger a response.
pub fn handle_fetch_entry(get_dht_data: FetchEntryData, context: Arc<Context>) {
    //CLEANUP, currently just using the old code from get to find the single content aspect
    // need to find all the other aspects too
    let address = Address::from(get_dht_data.entry_address.clone());
    let mut aspects = vec![];

    if let Ok(content) = get_content_aspect(&address, context.clone()) {
        aspects.push(content);


    } else {
        context.log(format!("warn/net/handle_fetch_entry: Could not get content aspect of requested entry {:?}", address));
    }

    let action_wrapper = ActionWrapper::new(Action::RespondFetch((get_dht_data, aspects)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}

fn get_content_aspect(entry_address: &Address, context: Arc<Context>) -> Result<EntryAspect, HolochainError> {
    let entry_with_meta = nucleus::actions::get_entry::get_entry_with_meta(&context, entry_address.clone())?
        .ok_or(HolochainError::EntryNotFoundLocally)?;

    let _ = entry_with_meta
        .entry
        .entry_type()
        .can_publish(&context)
        .ok_or(HolochainError::EntryIsPrivate)?;

    let headers = context
        .state()
        .expect("Could not get state for handle_fetch_entry")
        .get_headers(entry_address.clone())
        .map_err(|error| {
            let err_message = format!(
                "err/net/fetch/get_content_aspect: Error trying to get headers {:?}",
                error
            );
            context.log(err_message.clone());
            HolochainError::ErrorGeneric(err_message)
        })?;

    // TODO: this is just taking the first header..
    // We should actually transform all headers into EntryAspect::Headers and just the first one
    // into an EntryAspect content (What about ordering? Using the headers timestamp?)
    Ok(EntryAspect::Content(
        entry_with_meta.entry,
        headers[0].clone(),
    ))
}

/*
CLEANUP confirm that we really should never handle_fetch_entry_result (because we never send a fetch entry, only the network does)
/// The network comes back with a result to our previous GET request.
pub fn handle_fetch_entry_result(dht_data: FetchEntryResultData, context: Arc<Context>) {
    let action_wrapper = ActionWrapper::new(Action::HandleFetchResult(dht_data));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}
*/
