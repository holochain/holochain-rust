use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    entry::CanPublish,
    instance::dispatch_action,
    network::entry_aspect::EntryAspect,
    nucleus,
    workflows::get_entry_result::get_entry_with_meta_workflow,
};
use boolinator::*;
use holochain_core_types::{eav::Attribute, entry::Entry, error::HolochainError, time::Timeout};
use lib3h_protocol::data_types::FetchEntryData;
use holochain_persistence_api::cas::content::Address;
use std::sync::Arc;

/// The network has requested a DHT entry from us.
/// Lets try to get it and trigger a response.
pub fn handle_fetch_entry(get_dht_data: FetchEntryData, context: Arc<Context>) {
    let address = Address::from(get_dht_data.entry_address.clone());
    let mut aspects = vec![];

    match get_content_aspect(&address, context.clone()) {
        Ok(content_aspect) => {
            aspects.push(content_aspect);
            match get_meta_aspects(&address, context.clone()) {
                Ok(mut meta_aspects) => aspects.append(&mut meta_aspects),
                Err(get_meta_error) => {
                    context.log(format!(
                        "error/net/handle_fetch_entry: Error getting meta aspects for entry ({:?}), error: {:?}",
                        address,
                        get_meta_error,
                    ));
                }
            }
        }
        Err(get_content_error) => {
            context.log(format!(
                "warn/net/handle_fetch_entry: Could not get content aspect of requested entry ({:?}), error: {:?}",
                address,
                get_content_error,
            ));
        }
    }

    let action_wrapper = ActionWrapper::new(Action::RespondFetch((get_dht_data, aspects)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}

fn get_content_aspect(
    entry_address: &Address,
    context: Arc<Context>,
) -> Result<EntryAspect, HolochainError> {
    let entry_with_meta =
        nucleus::actions::get_entry::get_entry_with_meta(&context, entry_address.clone())?
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

fn get_meta_aspects(
    entry_address: &Address,
    context: Arc<Context>,
) -> Result<Vec<EntryAspect>, HolochainError> {
    let eavis = context
        .state()
        .expect("Could not get state for handle_fetch_entry")
        .dht()
        .get_all_metas(entry_address)?;

    let (aspects, errors): (Vec<_>, Vec<_>) = eavis
        .iter()
        .filter(|eavi| match eavi.attribute() {
            Attribute::LinkTag(_, _) => true,
            Attribute::RemovedLink(_, _) => true,
            Attribute::CrudLink => true,
            _ => false,
        })
        .map(|eavi| {
            let value_entry = context
                .block_on(get_entry_with_meta_workflow(
                    &context,
                    &eavi.value(),
                    &Timeout::default(),
                ))?
                .ok_or(HolochainError::from(
                    "Entry linked in EAV not found! This should never happen.",
                ))?;
            let header = value_entry.headers[0].to_owned();

            match eavi.attribute() {
                Attribute::LinkTag(_, _) => {
                    let link_data = unwrap_to!(value_entry.entry_with_meta.entry => Entry::LinkAdd);
                    Ok(EntryAspect::LinkAdd(link_data.clone(), header))
                }
                Attribute::RemovedLink(_, _) => {
                    let (link_data, removed_link_entries) =
                        unwrap_to!(value_entry.entry_with_meta.entry => Entry::LinkRemove);
                    Ok(EntryAspect::LinkRemove(
                        (link_data.clone(), removed_link_entries.clone()),
                        header,
                    ))
                }
                Attribute::CrudLink => Ok(EntryAspect::Update(
                    value_entry.entry_with_meta.entry,
                    header,
                )),
                _ => unreachable!(),
            }
        })
        .partition(Result::is_ok);

    if errors.len() > 0 {
        Err(errors[0].to_owned().err().unwrap())
    } else {
        Ok(aspects.into_iter().map(Result::unwrap).collect())
    }
}
