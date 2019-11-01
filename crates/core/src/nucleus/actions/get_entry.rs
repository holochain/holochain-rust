use crate::context::Context;
use holochain_core_types::{
    crud_status::CrudStatus,
    eav::{Attribute, EaviQuery, EntityAttributeValueIndex},
    entry::{Entry, EntryWithMeta},
    error::HolochainError,
};
use holochain_locksmith::RwLock;
use holochain_persistence_api::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    eav::IndexFilter,
};
use std::{collections::BTreeSet, str::FromStr, sync::Arc};

pub(crate) fn get_entry_from_cas(
    storage: &Arc<RwLock<dyn ContentAddressableStorage>>,
    address: &Address,
) -> Result<Option<Entry>, HolochainError> {
    if let Some(json) = (*storage.read().unwrap()).fetch(&address)? {
        let entry = Entry::try_from_content(&json)?;
        Ok(Some(entry))
    } else {
        Ok(None) // no errors but entry is not in CAS
    }
}

pub fn get_entry_from_agent_chain(
    context: &Arc<Context>,
    address: &Address,
) -> Result<Option<Entry>, HolochainError> {
    let agent = context.state().unwrap().agent();
    let top_header = agent.top_chain_header();
    let maybe_header = &agent
        .chain_store()
        .iter(&top_header)
        .find(|header| header.entry_address() == address);

    if maybe_header.is_none() {
        return Ok(None);
    }
    let cas = context
        .state()
        .unwrap()
        .agent()
        .chain_store()
        .content_storage();
    get_entry_from_cas(&cas.clone(), address)
}

pub(crate) fn get_entry_from_agent(
    context: &Arc<Context>,
    address: &Address,
) -> Result<Option<Entry>, HolochainError> {
    let cas = context
        .state()
        .unwrap()
        .agent()
        .chain_store()
        .content_storage();
    get_entry_from_cas(&cas.clone(), address)
}

pub(crate) fn get_entry_from_dht(
    context: &Arc<Context>,
    address: &Address,
) -> Result<Option<Entry>, HolochainError> {
    let cas = context.state().unwrap().dht().content_storage().clone();
    get_entry_from_cas(&cas, address)
}

pub(crate) fn get_entry_crud_meta_from_dht(
    context: &Arc<Context>,
    address: &Address,
) -> Result<Option<(CrudStatus, Option<Address>)>, HolochainError> {
    let state_dht = context.state().unwrap().dht().clone();
    let dht = state_dht.meta_storage().clone();

    let storage = &dht.clone();
    // Get crud-status
    let status_eavs = (*storage.read().unwrap()).fetch_eavi(&EaviQuery::new(
        Some(address.clone()).into(),
        Some(Attribute::CrudStatus).into(),
        None.into(),
        IndexFilter::LatestByAttribute,
        None,
    ))?;
    if status_eavs.len() == 0 {
        return Ok(None);
    }
    let mut crud_status = CrudStatus::Live;
    // TODO waiting for update/remove_eav() assert!(status_eavs.len() <= 1);
    // For now look for crud-status by life-cycle order: Deleted, Modified, Live
    let has_deleted = status_eavs
        .clone()
        .into_iter()
        .filter(|e| {
            CrudStatus::from_str(String::from(e.value()).as_ref()) == Ok(CrudStatus::Deleted)
        })
        .collect::<BTreeSet<EntityAttributeValueIndex>>()
        .len()
        > 0;
    if has_deleted {
        crud_status = CrudStatus::Deleted;
    } else {
        let has_modified = status_eavs
            .into_iter()
            .filter(|e| {
                CrudStatus::from_str(String::from(e.value()).as_ref()) == Ok(CrudStatus::Modified)
            })
            .collect::<BTreeSet<EntityAttributeValueIndex>>()
            .len()
            > 0;
        if has_modified {
            crud_status = CrudStatus::Modified;
        }
    }
    // Get crud-link
    let mut maybe_link_update_delete = None;
    let link_eavs = (*storage.read().unwrap()).fetch_eavi(&EaviQuery::new(
        Some(address.clone()).into(),
        Some(Attribute::CrudLink).into(),
        None.into(),
        IndexFilter::LatestByAttribute,
        None,
    ))?;
    assert!(
        link_eavs.len() <= 1,
        "link_eavs.len() = {}",
        link_eavs.len()
    );
    if link_eavs.len() == 1 {
        maybe_link_update_delete = Some(link_eavs.iter().next().unwrap().value());
    }
    // Done
    Ok(Some((crud_status, maybe_link_update_delete)))
}

pub fn get_entry_with_meta(
    context: &Arc<Context>,
    address: Address,
) -> Result<Option<EntryWithMeta>, HolochainError> {
    // 1. try to get the entry
    let entry = match get_entry_from_dht(context, &address) {
        Err(err) => return Err(err),
        Ok(None) => return Ok(None),
        Ok(Some(entry)) => entry,
    };

    // 2. try to get the entry's metadata
    let (crud_status, maybe_link_update_delete) =
        match get_entry_crud_meta_from_dht(context, &address)? {
            Some(crud_info) => crud_info,
            None => return Ok(None), //If we cannot get the CRUD status for above entry it is not an
                                     //entry that is held by this DHT. It might be in the DHT CAS
                                     //because DHT and chain share the same CAS or it maybe just got
                                     //added by a concurrent process but the CRUD status is still about
                                     //to get set. Either way, we should treat it as not existent (yet).
        };
    let item = EntryWithMeta {
        entry,
        crud_status,
        maybe_link_update_delete,
    };
    Ok(Some(item))
}

#[cfg(test)]
pub mod tests {
    use crate::instance::tests::test_context_with_state;
    use holochain_core_types::entry::test_entry;
    use holochain_persistence_api::cas::content::AddressableContent;

    #[test]
    fn test_get_entry_from_dht_cas() {
        let entry = test_entry();
        let context = test_context_with_state(None);
        let result = super::get_entry_from_dht(&context, &entry.address());
        assert_eq!(Ok(None), result);
        let storage = &context.state().unwrap().dht().content_storage().clone();
        (*storage.write().unwrap()).add(&entry).unwrap();
        let result = super::get_entry_from_dht(&context, &entry.address());
        assert_eq!(Ok(Some(entry.clone())), result);
    }
    /*
        #[test]
        fn test_get_entry_from_agent_chain() {
    // write this test when its easier to get a mutable agent state
        }
    */
}
