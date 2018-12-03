extern crate serde_json;
use crate::context::Context;
use futures::future::{self, FutureObj};
use holochain_core_types::{
    cas::content::Address,
    crud_status::{CrudStatus, LINK_NAME, STATUS_NAME},
    eav::EntityAttributeValue,
    entry::Entry,
    error::HolochainError,
};
use holochain_wasm_utils::api_serialization::get_entry::{
    GetEntryArgs, GetEntryOptions, GetEntryResult, StatusRequestKind,
};
use std::{collections::HashSet, convert::TryInto, sync::Arc};

pub(crate) fn get_entry_from_dht_cas(
    context: &Arc<Context>,
    address: Address,
) -> Result<Option<Entry>, HolochainError> {
    let dht = context.state().unwrap().dht().content_storage();
    let storage = &dht.clone();
    let json = (*storage.read().unwrap()).fetch(&address)?;
    let entry: Option<Entry> = json
        .and_then(|js| js.try_into().ok())
        .map(|s: Entry| s.into());
    Ok(entry)
}

pub(crate) fn get_entry_meta_from_dht(
    context: &Arc<Context>,
    address: Address,
) -> Result<Option<(CrudStatus, Option<Address>)>, HolochainError> {
    let dht = context.state().unwrap().dht().meta_storage();
    let storage = &dht.clone();
    // Get crud-status
    let status_eavs = (*storage.read().unwrap()).fetch_eav(
        Some(address.clone()),
        Some(STATUS_NAME.to_string()),
        None,
    )?;
    if status_eavs.len() == 0 {
        return Ok(None);
    }
    let mut crud_status = CrudStatus::LIVE;
    // TODO waiting for update/remove_eav() assert!(status_eavs.len() <= 1);
    // For now look for crud-status by life-cycle order: DELETED, MODIFIED, LIVE
    let has_deleted = status_eavs
        .iter()
        .filter(|e| CrudStatus::from(String::from(e.value())) == CrudStatus::DELETED)
        .collect::<HashSet<&EntityAttributeValue>>()
        .len()
        > 0;
    if has_deleted {
        crud_status = CrudStatus::DELETED;
    } else {
        let has_modified = status_eavs
            .iter()
            .filter(|e| CrudStatus::from(String::from(e.value())) == CrudStatus::MODIFIED)
            .collect::<HashSet<&EntityAttributeValue>>()
            .len()
            > 0;
        if has_modified {
            crud_status = CrudStatus::MODIFIED;
        }
    }
    // Get crud-link
    let mut maybe_crud_link = None;
    let link_eavs =
        (*storage.read().unwrap()).fetch_eav(Some(address), Some(LINK_NAME.to_string()), None)?;
    assert!(link_eavs.len() <= 1);
    if link_eavs.len() == 1 {
        maybe_crud_link = Some(link_eavs.iter().next().unwrap().value());
    }
    // Done
    Ok(Some((crud_status, maybe_crud_link)))
}

/// GetEntry Action Creator
///
/// Returns a future that resolves to an Ok(GetEntryResult) or an Err(HolochainError).
pub fn get_entry<'a>(
    context: &'a Arc<Context>,
    args: &GetEntryArgs,
) -> FutureObj<'a, Result<GetEntryResult, HolochainError>> {
    let mut entry_result = GetEntryResult::new();
    match get_entry_rec(
        context,
        &mut entry_result,
        args.address.clone(),
        args.options.clone(),
    ) {
        Err(err) => FutureObj::new(Box::new(future::err(err))),
        Ok(_) => FutureObj::new(Box::new(future::ok(entry_result))),
    }
}

/// Recursive function for filling GetEntryResult by walking the crud-links
pub fn get_entry_rec<'a>(
    context: &'a Arc<Context>,
    entry_result: &mut GetEntryResult,
    address: Address,
    options: GetEntryOptions,
) -> Result<(), HolochainError> {
    // 1. try to get the Entry
    let address = address.clone();
    let maybe_entry = get_entry_from_dht_cas(context, address.clone())?;
    // No entry = return empty result
    if maybe_entry.is_none() {
        return Ok(());
    }
    let entry = maybe_entry.unwrap();
    // 2. try to get entry's meta
    let meta = get_entry_meta_from_dht(context, address.clone())?;
    let meta = meta.expect("Entry should have crud-status metadata");
    // 3. Add Entry + Meta to GetEntryResult
    entry_result.addresses.push(address.clone());
    entry_result.entries.push(entry);
    entry_result.crud_status.push(meta.0);
    if let Some(new_address) = meta.1 {
        entry_result.crud_links.insert(address, new_address.clone());
        // Don't follow link if its a DeletionEntry
        if meta.0 != CrudStatus::DELETED {
            // 4. Follow link depending on StatusRequestKind
            match options.status_request {
                StatusRequestKind::Initial => {}
                StatusRequestKind::Latest => {
                    *entry_result = GetEntryResult::new();
                    get_entry_rec(context, entry_result, new_address, options)?;
                }
                StatusRequestKind::All => {
                    get_entry_rec(context, entry_result, new_address, options)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use crate::instance::tests::test_context_with_state;
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::AddressableContent,
        crud_status::{create_crud_status_eav, CrudStatus},
        entry::test_entry,
    };
    use holochain_wasm_utils::api_serialization::get_entry::*;

    #[test]
    fn get_entry_from_dht_cas() {
        let entry = test_entry();
        let context = test_context_with_state();
        let result = super::get_entry_from_dht_cas(&context, entry.address());
        assert_eq!(Ok(None), result);
        let storage = &context.state().unwrap().dht().content_storage().clone();
        (*storage.write().unwrap()).add(&entry).unwrap();
        let result = super::get_entry_from_dht_cas(&context, entry.address());
        assert_eq!(Ok(Some(entry.clone())), result);
    }

    #[test]
    fn get_entry_futures() {
        let entry = test_entry();
        let context = test_context_with_state();
        let args = GetEntryArgs {
            address: entry.address(),
            options: GetEntryOptions {
                status_request: StatusRequestKind::Latest,
            },
        };
        let future = super::get_entry(&context, &args);
        let res = block_on(future);
        assert_eq!(0, res.unwrap().entries.len());
        let content_storage = &context.state().unwrap().dht().content_storage().clone();
        (*content_storage.write().unwrap()).add(&entry).unwrap();
        let status_eav = create_crud_status_eav(&entry.address(), CrudStatus::LIVE);
        let meta_storage = &context.state().unwrap().dht().meta_storage().clone();
        (*meta_storage.write().unwrap())
            .add_eav(&status_eav)
            .unwrap();
        let future = super::get_entry(&context, &args);
        let res = block_on(future);
        let entry_result = res.unwrap();
        assert_eq!(&entry, entry_result.entries.iter().next().unwrap());
    }

}
