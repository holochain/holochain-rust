extern crate serde_json;
use crate::context::Context;
use holochain_core_types::{
    cas::content::Address,
    crud_status::{CrudStatus, LINK_NAME, STATUS_NAME},
    eav::EntityAttributeValue,
    entry::{Entry, EntryWithMeta},
    error::HolochainError,
};

use std::{collections::HashSet, convert::TryInto, str::FromStr, sync::Arc};

pub(crate) fn get_entry_from_dht(
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

pub(crate) fn get_entry_crud_meta_from_dht(
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
    let mut crud_status = CrudStatus::Live;
    // TODO waiting for update/remove_eav() assert!(status_eavs.len() <= 1);
    // For now look for crud-status by life-cycle order: Deleted, Modified, Live
    let has_deleted = status_eavs
        .iter()
        .filter(|e| {
            CrudStatus::from_str(String::from(e.value()).as_ref()) == Ok(CrudStatus::Deleted)
        })
        .collect::<HashSet<&EntityAttributeValue>>()
        .len()
        > 0;
    if has_deleted {
        crud_status = CrudStatus::Deleted;
    } else {
        let has_modified = status_eavs
            .iter()
            .filter(|e| {
                CrudStatus::from_str(String::from(e.value()).as_ref()) == Ok(CrudStatus::Modified)
            })
            .collect::<HashSet<&EntityAttributeValue>>()
            .len()
            > 0;
        if has_modified {
            crud_status = CrudStatus::Modified;
        }
    }
    // Get crud-link
    let mut maybe_crud_link = None;
    let link_eavs =
        (*storage.read().unwrap()).fetch_eav(Some(address), Some(LINK_NAME.to_string()), None)?;
    assert!(
        link_eavs.len() <= 1,
        "link_eavs.len() = {}",
        link_eavs.len()
    );
    if link_eavs.len() == 1 {
        maybe_crud_link = Some(link_eavs.iter().next().unwrap().value());
    }
    // Done
    Ok(Some((crud_status, maybe_crud_link)))
}

/// GetEntry Action Creator
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub fn get_entry_with_meta<'a>(
    context: &'a Arc<Context>,
    address: Address,
) -> Result<Option<EntryWithMeta>, HolochainError> {
    // 1. try to get the entry
    let entry = match get_entry_from_dht(context, address.clone()) {
        Err(err) => return Err(err),
        Ok(None) => return Ok(None),
        Ok(Some(entry)) => entry,
    };
    // 2. try to get the entry's metadata
    let maybe_meta = get_entry_crud_meta_from_dht(context, address.clone());
    if let Err(err) = maybe_meta {
        return Err(err);
    }
    let (crud_status, maybe_crud_link) = maybe_meta
        .unwrap()
        .expect("Entry should have crud-status metadata");
    let item = EntryWithMeta {
        entry,
        crud_status,
        maybe_crud_link,
    };
    Ok(Some(item))
}

#[cfg(test)]
pub mod tests {
    use crate::instance::tests::test_context_with_state;
    use holochain_core_types::{cas::content::AddressableContent, entry::test_entry};

    #[test]
    fn get_entry_from_dht_cas() {
        let entry = test_entry();
        let context = test_context_with_state(None);
        let result = super::get_entry_from_dht(&context, entry.address());
        assert_eq!(Ok(None), result);
        let storage = &context.state().unwrap().dht().content_storage().clone();
        (*storage.write().unwrap()).add(&entry).unwrap();
        let result = super::get_entry_from_dht(&context, entry.address());
        assert_eq!(Ok(Some(entry.clone())), result);
    }
}
