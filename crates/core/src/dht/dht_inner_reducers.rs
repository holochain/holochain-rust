///
/// Inner DHT reducers are not pure functions but rather functions designed to make the required
/// mutations to a newly cloned DhtState object. Unlike the reducers they do not need a specific signature.
/// The should have a signature similar to
///
/// `reduce_some_thing_inner(store: &mut DhtStore, <other required data>) -> HcResult<someReturnType>`
///
/// It is up to the calling reducer function whether the new state object should be kept and what to do with the return value
///
use crate::dht::dht_store::DhtStore;
use crate::{
    content_store::{AddContent, GetContent},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::{
    crud_status::{create_crud_link_eav, create_crud_status_eav, CrudStatus},
    eav::{Attribute, EaviQuery, EntityAttributeValueIndex},
    entry::Entry,
    error::{HcResult, HolochainError},
    link::Link,
};

use holochain_persistence_api::{
    cas::content::{Address, AddressableContent},
    eav::IndexFilter,
};

use std::{collections::BTreeSet, str::FromStr};

pub(crate) enum LinkModification {
    Add,
    Remove,
}

/// Used as the inner function for both commit and hold reducers
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub(crate) fn reduce_store_entry_inner(store: &mut DhtStore, entry: &Entry) -> HcResult<()> {
    match store.add(entry) {
        Ok(()) => create_crud_status_eav(&entry.address(), CrudStatus::Live).map(|status_eav| {
            store.add_eavi(&status_eav).map(|_| ()).map_err(|e| {
                format!("err/dht: dht::reduce_store_entry_inner() FAILED {:?}", e).into()
            })
        })?,
        Err(e) => Err(format!("err/dht: dht::reduce_store_entry_inner() FAILED {:?}", e).into()),
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub(crate) fn reduce_add_remove_link_inner(
    store: &mut DhtStore,
    link: &Link,
    address: &Address,
    link_modification: LinkModification,
) -> HcResult<Address> {
    if store.contains(link.base())? {
        let attr = match link_modification {
            LinkModification::Add => {
                Attribute::LinkTag(link.link_type().to_string(), link.tag().to_string())
            }
            LinkModification::Remove => {
                Attribute::RemovedLink(link.link_type().to_string(), link.tag().to_string())
            }
        };
        let eav = EntityAttributeValueIndex::new(link.base(), &attr, address)?;
        store.add_eavi(&eav)?;
        Ok(link.base().clone())
    } else {
        Err(HolochainError::ErrorGeneric(String::from(
            "Base for link not found",
        )))
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub(crate) fn reduce_update_entry_inner(
    store: &mut DhtStore,
    old_address: &Address,
    new_address: &Address,
) -> HcResult<Address> {
    // Update crud-status
    let new_status_eav = create_crud_status_eav(old_address, CrudStatus::Modified)?;
    store.add_eavi(&new_status_eav)?;
    // add link from old to new
    let crud_link_eav = create_crud_link_eav(old_address, new_address)?;
    store.add_eavi(&crud_link_eav)?;

    Ok(new_address.clone())
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub(crate) fn reduce_remove_entry_inner(
    store: &mut DhtStore,
    latest_deleted_address: &Address,
    deletion_address: &Address,
) -> HcResult<Address> {
    let entry = store
        .get(latest_deleted_address)?
        .ok_or_else(|| HolochainError::ErrorGeneric("trying to remove a missing entry".into()))?;

    // pre-condition: entry_type must not be sys type, since they cannot be deleted
    if entry.entry_type().is_sys() {
        return Err(HolochainError::ErrorGeneric(
            "trying to remove a system entry type".into(),
        ));
    }
    // pre-condition: Current status must be Live
    // get current status
    let status_eavs = store.fetch_eavi(&EaviQuery::new(
        Some(latest_deleted_address.clone()).into(),
        Some(Attribute::CrudStatus).into(),
        None.into(),
        IndexFilter::LatestByAttribute,
        None,
    ))?;

    //TODO clean up some of the early returns in this
    // TODO waiting for update/remove_eav() assert!(status_eavs.len() <= 1);
    // For now checks if crud-status other than Live are present
    let status_eavs = status_eavs
        .into_iter()
        .filter(|e| CrudStatus::from_str(String::from(e.value()).as_ref()) != Ok(CrudStatus::Live))
        .collect::<BTreeSet<EntityAttributeValueIndex>>();

    if !status_eavs.is_empty() {
        return Err(HolochainError::ErrorGeneric(
            "entry_status != CrudStatus::Live".into(),
        ));
    }
    // Update crud-status
    let new_status_eav = create_crud_status_eav(latest_deleted_address, CrudStatus::Deleted)
        .map_err(|_| HolochainError::ErrorGeneric("Could not create eav".into()))?;
    store.add_eavi(&new_status_eav)?;

    // Update crud-link
    let crud_link_eav = create_crud_link_eav(latest_deleted_address, deletion_address)
        .map_err(|_| HolochainError::ErrorGeneric(String::from("Could not create eav")))?;
    store.add_eavi(&crud_link_eav)?;

    Ok(latest_deleted_address.clone())
}
