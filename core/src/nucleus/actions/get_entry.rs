extern crate serde_json;
use context::Context;
use futures::{future, Future};
use holochain_core_types::{
    cas::content::Address,
    entry::{Entry, SerializedEntry},
    error::HolochainError,
    eav::EntityAttributeValue,
    crud_status::{CrudStatus, STATUS_NAME},
};
use std::{convert::TryInto, sync::Arc};
use holochain_wasm_utils::api_serialization::get_entry::{GetEntryResult, GetEntryArgs};
use std::collections::HashMap;
use std::collections::HashSet;

pub(crate) fn get_entry_from_dht_cas(
    context: &Arc<Context>,
    address: Address,
) -> Result<Option<Entry>, HolochainError> {
    let dht = context.state().unwrap().dht().content_storage();
    let storage = &dht.clone();
    let json = (*storage.read().unwrap()).fetch(&address)?;
    let entry: Option<Entry> = json
        .and_then(|js| js.try_into().ok())
        .map(|s: SerializedEntry| s.into());
    Ok(entry)
}

pub(crate) fn get_entry_meta_from_dht(
    context: &Arc<Context>,
    address: Address,
) -> Result<Option<(CrudStatus, Option<Address>)>, HolochainError> {
    let dht = context.state().unwrap().dht().meta_storage();
    let storage = &dht.clone();
    let status_eavs =
        (*storage.read().unwrap()).fetch_eav(Some(address), Some(STATUS_NAME.to_string()), None)?;
    if status_eavs.len() == 0 {
        return Ok(None);
    }
    // FIXME waiting for update/remove_eav() assert!(status_eavs.len() <= 1);
    let has_deleted = status_eavs
        .iter()
        .filter(|e| CrudStatus::from(String::from(e.value())) == CrudStatus::DELETED)
        .collect::<HashSet<&EntityAttributeValue>>().len() > 0;
    if has_deleted {
        return Ok(Some((CrudStatus::DELETED, None)));
    }
    let has_modified = status_eavs
        .iter()
        .filter(|e| CrudStatus::from(String::from(e.value())) == CrudStatus::MODIFIED)
        .collect::<HashSet<&EntityAttributeValue>>().len() > 0;
    if has_modified {
        return Ok(Some((CrudStatus::MODIFIED, None)));
    }
    //let status = CrudStatus::from(String::from(status_eav.value()));
    Ok(Some((CrudStatus::LIVE, None)))
}

/// GetEntry Action Creator
///
/// Returns a future that resolves to an Ok(GetEntryResult) or an Err(HolochainError).
pub fn get_entry(
    context: &Arc<Context>,
    args: &GetEntryArgs,
) -> Box<dyn Future<Item = GetEntryResult, Error = HolochainError>> {
    // First try to get the Entry
    let address = args.address.clone();
    let res = get_entry_from_dht_cas(context, address.clone());
    let maybe_entry = match res {
        Err(err) => return Box::new(future::err(err)),
        Ok(result) => result,
    };
    // No entry = return empty result
    if maybe_entry.is_none() {
        return Box::new(future::ok(GetEntryResult::new()));
    }
    let entry = maybe_entry.unwrap();
    // Second try to get entry's meta
    let res = get_entry_meta_from_dht(context, address.clone());
    let meta = match res {
        Err(err) => return Box::new(future::err(err)),
        Ok(result) => result.expect("Entry should have meta"),
    };
    // Create GetEntryResult for just one Entry
    let entry_result = GetEntryResult {
        addresses: vec![address],
        entries: vec![entry.serialize()],
        crud_status: vec![meta.0],
        crud_links: HashMap::new(), // FIXME put link here if found
    };
    println!("\n get_entry: {:?}\n", entry_result);
    Box::new(future::ok(entry_result))
}

#[cfg(test)]
pub mod tests {
    use futures::executor::block_on;
    use holochain_core_types::{cas::content::AddressableContent, entry::test_entry};
    use instance::tests::test_context_with_state;

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
    fn get_entry_from_dht_cas_futures() {
        let entry = test_entry();
        let context = test_context_with_state();
        let future = super::get_entry(&context, entry.address());
        assert_eq!(Ok(None), block_on(future));
        let storage = &context.state().unwrap().dht().content_storage().clone();
        (*storage.write().unwrap()).add(&entry).unwrap();
        let future = super::get_entry(&context, entry.address());
        assert_eq!(Ok(Some(entry.clone())), block_on(future));
    }

}
