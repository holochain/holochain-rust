use crate::{context::Context, network, nucleus};

use futures::executor::block_on;
use holochain_core_types::{
    cas::content::Address,
    crud_status::CrudStatus,
    entry::{Entry, EntryWithMeta},
    error::HolochainError,
};
use holochain_wasm_utils::api_serialization::get_entry::{
    EntryHistory, GetEntryArgs, GetEntryOptions, StatusRequestKind,
};
use std::sync::Arc;

///// Get Entry workflow
//pub async fn get_entry_with_meta_workflow<'a>(
//    context: &'a Arc<Context>,
//    address: &'a Address,
//) -> Result<Option<EntryWithMeta>, HolochainError> {
//    // 1. Try to get the entry locally (i.e. local DHT shard)
//    let maybe_entry_with_meta = await!(nucleus::actions::get_entry::get_entry_with_meta(
//        context,
//        address.clone(),
//    ))?;
//    if maybe_entry_with_meta.is_some() {
//        return Ok(maybe_entry_with_meta.unwrap());
//    }
//    // 2. No result, so try on the network
//    await!(network::actions::get_entry::get_entry(context, &address.clone()))
//}

/// Get EntryHistory workflow
pub fn get_entry_history_workflow<'a>(
    context: &'a Arc<Context>,
    args: &'a GetEntryArgs,
) -> Result<EntryHistory, HolochainError> {
    // Initiate the recursive look-up of entries
    let mut entry_history = EntryHistory::new();
    let res = get_entry_rec(
        context,
        &mut entry_history,
        args.address.clone(),
        args.options.clone(),
    );
    // Return entry_result
    res.map(|_| entry_history)
}

/// Recursive function for filling GetEntryResult by walking the crud-links.
/// Result is accumulateed in the `entry_result` argument.
pub fn get_entry_rec<'a>(
    context: &'a Arc<Context>,
    entry_history: &'a mut EntryHistory,
    address: Address,
    options: GetEntryOptions,
) -> Result<(), HolochainError> {
    // 1a. Try to get the entry locally (i.e. local DHT shard)

    let future = nucleus::actions::get_entry::get_entry_with_meta(context, address.clone());
    let maybe_entry_with_meta = block_on(future)?;
    let entry_with_meta = if maybe_entry_with_meta.is_some() {
        maybe_entry_with_meta.unwrap()
    } else {
        // 1b. No result, so try on the network
        let future = network::actions::get_entry::get_entry(context, &address);
        let maybe_entry_with_meta = block_on(future)?;
        if maybe_entry_with_meta.is_none() {
            // No entry found => exit
            return Ok(());
        }
        maybe_entry_with_meta.unwrap()
    };
    // 1. try to get the complete-entry locally and globally
    //    let res = await!(get_entry_with_meta_workflow(context, &address.clone()))?;
    //    if let Err(err) = res {
    //        return Err(err);
    //    }
    // 2. Add complete-entry to GetEntryResult
    entry_history.push(&entry_with_meta);
    // 3. Check if there is a crud-link to follow
    if entry_with_meta.maybe_crud_link.is_none()
        || entry_with_meta.crud_status == CrudStatus::DELETED
        || options.status_request == StatusRequestKind::Initial
    {
        return Ok(());
    }
    let new_address = entry_with_meta.maybe_crud_link.unwrap();
    // 4. Follow crud-link depending on StatusRequestKind
    match options.status_request {
        StatusRequestKind::Initial => unreachable!(),
        StatusRequestKind::Latest => {
            *entry_history = EntryHistory::new();
            get_entry_rec(context, entry_history, new_address, options)
        }
        StatusRequestKind::All => get_entry_rec(context, entry_history, new_address, options),
    }
}
