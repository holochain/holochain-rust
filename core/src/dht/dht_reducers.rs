//! all DHT reducers

use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::dht_store::DhtStore,
    network::entry_with_header::EntryWithHeader,
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    crud_status::{create_crud_link_eav, create_crud_status_eav, CrudStatus},
    eav::{Attribute, EaviQuery, EntityAttributeValueIndex, IndexFilter},
    entry::Entry,
    error::HolochainError,
};
use std::{collections::BTreeSet, convert::TryFrom, str::FromStr, sync::Arc};

// A function that might return a mutated DhtStore
type DhtReducer = fn(Arc<Context>, &DhtStore, &ActionWrapper) -> Option<DhtStore>;

/// DHT state-slice Reduce entry point.
/// Note: Can't block when dispatching action here because we are inside the reduce's mutex
pub fn reduce(
    context: Arc<Context>,
    old_store: Arc<DhtStore>,
    action_wrapper: &ActionWrapper,
) -> Arc<DhtStore> {
    // Get reducer
    let maybe_reducer = resolve_reducer(action_wrapper);
    if maybe_reducer.is_none() {
        return old_store;
    }
    let reducer = maybe_reducer.unwrap();
    // Reduce
    let store = old_store.clone();
    let maybe_new_store = reducer(context, &store, &action_wrapper);
    match maybe_new_store {
        None => old_store,
        Some(new_store) => Arc::new(new_store),
    }
}

/// Maps incoming action to the correct reducer
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<DhtReducer> {
    match action_wrapper.action() {
        Action::Commit(_) => Some(reduce_hold_entry),
        Action::Hold(_) => Some(reduce_hold_entry),
        Action::UpdateEntry(_) => Some(reduce_update_entry),
        Action::RemoveEntry(_) => Some(reduce_remove_entry),
        Action::AddLink(_) => Some(reduce_add_link),
        Action::RemoveLink(_) => Some(reduce_remove_link),
        _ => None,
    }
}

pub(crate) fn reduce_hold_entry(
    context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    match action_wrapper.action().clone() {
        Action::Commit((entry, _, _)) => reduce_store_entry_common(context, old_store, &entry),
        Action::Hold(EntryWithHeader { entry, header }) => {
            reduce_store_entry_common(context.clone(), old_store, &entry).and_then(|state| {
                state.add_header_for_entry(&entry, &header).ok()?;
                Some(state)
            })
        }
        _ => unreachable!(),
    }
}

fn reduce_store_entry_common(
    context: Arc<Context>,
    old_store: &DhtStore,
    entry: &Entry,
) -> Option<DhtStore> {
    // Add it to local storage
    let new_store = (*old_store).clone();
    let content_storage = &new_store.content_storage().clone();
    let res = (*content_storage.write().unwrap()).add(entry).ok();
    if res.is_some() {
        let meta_storage = &new_store.meta_storage().clone();
        create_crud_status_eav(&entry.address(), CrudStatus::Live)
            .map(|status_eav| {
                let meta_res = (*meta_storage.write().unwrap()).add_eavi(&status_eav);
                meta_res
                    .map(|_| Some(new_store))
                    .map_err(|err| {
                        context.log(format!(
                            "err/dht: reduce_hold_entry: meta_storage write failed!: {:?}",
                            err
                        ));
                        None::<DhtStore>
                    })
                    .ok()
                    .unwrap_or(None)
            })
            .ok()
            .unwrap_or(None)
    } else {
        context.log(format!(
            "err/dht: dht::reduce_hold_entry() FAILED {:?}",
            res
        ));
        None
    }
}

//
pub(crate) fn reduce_add_link(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // Get Action's input data
    let action = action_wrapper.action();
    let link = unwrap_to!(action => Action::AddLink);

    let mut new_store = (*old_store).clone();
    let storage = &old_store.content_storage().clone();
    if !(*storage.read().unwrap()).contains(link.base()).unwrap() {
        new_store.actions_mut().insert(
            action_wrapper.clone(),
            Err(HolochainError::ErrorGeneric(String::from(
                "Base for link not found",
            ))),
        );
        Some(new_store)
    } else {
        let eav = EntityAttributeValueIndex::new(
            link.base(),
            &Attribute::LinkTag(link.tag().to_owned()),
            link.target(),
        );
        eav.map(|e| {
            let storage = new_store.meta_storage();
            let result = storage.write().unwrap().add_eavi(&e);
            new_store
                .actions_mut()
                .insert(action_wrapper.clone(), result.map(|_| link.base().clone()));
            Some(new_store)
        })
        .ok()
        .unwrap_or(None)
    }
}

pub(crate) fn reduce_remove_link(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // Get Action's input data
    let action = action_wrapper.action();
    let link = unwrap_to!(action => Action::RemoveLink);
    let mut new_store = (*old_store).clone();
    let storage = &old_store.content_storage().clone();
    if !(*storage.read().unwrap()).contains(link.base()).unwrap() {
        new_store.actions_mut().insert(
            action_wrapper.clone(),
            Err(HolochainError::ErrorGeneric(String::from(
                "Base for link not found for remove",
            ))),
        );
        Some(new_store)
    } else {
        let eav = EntityAttributeValueIndex::new(
            link.base(),
            &Attribute::RemovedLink(link.tag().to_string()),
            link.target(),
        );
        eav.map(|e| {
            let storage = new_store.meta_storage();
            let result = storage.write().unwrap().add_eavi(&e);
            new_store
                .actions_mut()
                .insert(action_wrapper.clone(), result.map(|_| link.base().clone()));
            Some(new_store)
        })
        .ok()
        .unwrap_or(None)
    }
}

//
pub(crate) fn reduce_update_entry(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // Setup
    let action = action_wrapper.action();
    let (old_address, new_address) = unwrap_to!(action => Action::UpdateEntry);
    let mut new_store = (*old_store).clone();
    // Update crud-status
    let latest_old_address = old_address;
    let meta_storage = &new_store.meta_storage().clone();
    let closure_store = new_store.clone();
    let new_status_eav_option = create_crud_status_eav(latest_old_address, CrudStatus::Modified)
        .map(|new_status_eav| {
            let res = (*meta_storage.write().unwrap()).add_eavi(&new_status_eav);
            res.map(|_| None)
                .map_err(|err| {
                    closure_store
                        .clone()
                        .actions_mut()
                        .insert(action_wrapper.clone(), Err(err));
                    Some(closure_store.clone())
                })
                .ok()
                .unwrap_or(Some(closure_store.clone()))
        })
        .ok()
        .unwrap_or(None);
    if new_status_eav_option.is_some() {
        new_status_eav_option
    } else {
        // Update crud-link
        create_crud_link_eav(latest_old_address, new_address)
            .map(|crud_link_eav| {
                let res = (*meta_storage.write().unwrap()).add_eavi(&crud_link_eav);
                let res_option = res.clone().ok();
                res_option
                    .and_then(|_| {
                        new_store.actions_mut().insert(
                            action_wrapper.clone(),
                            res.clone().map(|_| new_address.clone()),
                        );
                        Some(new_store.clone())
                    })
                    .or_else(|| {
                        new_store
                            .actions_mut()
                            .insert(action_wrapper.clone(), Err(res.err().unwrap()));
                        Some(new_store.clone())
                    })
            })
            .ok()
            .unwrap_or(None)
    }
}

pub(crate) fn reduce_remove_entry(
    context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // Setup
    let action = action_wrapper.action();
    let (deleted_address, deletion_address) = unwrap_to!(action => Action::RemoveEntry);
    let mut new_store = (*old_store).clone();
    // Act
    let res = reduce_remove_entry_inner(context, &mut new_store, deleted_address, deletion_address);
    // Done
    new_store.actions_mut().insert(action_wrapper.clone(), res);
    Some(new_store)
}

//
fn reduce_remove_entry_inner(
    _context: Arc<Context>,
    new_store: &mut DhtStore,
    latest_deleted_address: &Address,
    deletion_address: &Address,
) -> Result<Address, HolochainError> {
    // pre-condition: Must already have entry in local content_storage
    let content_storage = &new_store.content_storage().clone();

    let maybe_json_entry = content_storage
        .read()
        .unwrap()
        .fetch(latest_deleted_address)
        .unwrap();
    let json_entry = maybe_json_entry.ok_or_else(|| {
        HolochainError::ErrorGeneric(String::from("trying to remove a missing entry"))
    })?;

    let entry = Entry::try_from(json_entry).expect("Stored content should be a valid entry.");
    // pre-condition: entry_type must not by sys type, since they cannot be deleted
    if entry.entry_type().to_owned().is_sys() {
        return Err(HolochainError::ErrorGeneric(String::from(
            "trying to remove a system entry type",
        )));
    }
    // pre-condition: Current status must be Live
    // get current status
    let meta_storage = &new_store.meta_storage().clone();
    let status_eavs = meta_storage.read().unwrap().fetch_eavi(&EaviQuery::new(
        Some(latest_deleted_address.clone()).into(),
        Some(Attribute::CrudStatus).into(),
        None.into(),
        IndexFilter::LatestByAttribute,
    ))?;

    //TODO clean up some of the early returns in this
    // TODO waiting for update/remove_eav() assert!(status_eavs.len() <= 1);
    // For now checks if crud-status other than Live are present
    let status_eavs = status_eavs
        .into_iter()
        .filter(|e| CrudStatus::from_str(String::from(e.value()).as_ref()) != Ok(CrudStatus::Live))
        .collect::<BTreeSet<EntityAttributeValueIndex>>();
    if !status_eavs.is_empty() {
        return Err(HolochainError::ErrorGeneric(String::from(
            "entry_status != CrudStatus::Live",
        )));
    }
    // Update crud-status
    let result = create_crud_status_eav(latest_deleted_address, CrudStatus::Deleted);
    if result.is_err() {
        return Err(HolochainError::ErrorGeneric(String::from(
            "Could not create eav",
        )));
    }
    let new_status_eav = result.expect("should unwrap eav");
    let meta_storage = &new_store.meta_storage().clone();

    (*meta_storage.write().unwrap()).add_eavi(&new_status_eav)?;

    // Update crud-link
    let crud_link_eav = create_crud_link_eav(latest_deleted_address, deletion_address)
        .map_err(|_| HolochainError::ErrorGeneric(String::from("Could not create eav")))?;
    let res = (*meta_storage.write().unwrap()).add_eavi(&crud_link_eav);

    res.map(|_| latest_deleted_address.clone())
}

//
#[allow(dead_code)]
pub(crate) fn reduce_get_links(
    _context: Arc<Context>,
    _old_store: &DhtStore,
    _action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // FIXME
    None
}

#[cfg(test)]
pub mod tests {

    use crate::{
        action::{Action, ActionWrapper},
        dht::{
            dht_reducers::{reduce, reduce_hold_entry},
            dht_store::DhtStore,
        },
        instance::tests::test_context,
        network::entry_with_header::EntryWithHeader,
        state::test_store,
    };
    use holochain_core_types::{
        cas::content::AddressableContent,
        chain_header::test_chain_header,
        eav::{Attribute, EavFilter, EaviQuery, IndexFilter},
        entry::{test_entry, test_sys_entry, Entry},
        link::Link,
    };
    use std::{
        convert::TryFrom,
        sync::{Arc, RwLock},
    };

    #[test]
    fn reduce_hold_entry_test() {
        let context = test_context("bob", None);
        let store = test_store(context.clone());

        // test_entry is not sys so should do nothing
        let storage = &store.dht().content_storage().clone();

        let sys_entry = test_sys_entry();
        let entry_wh = EntryWithHeader {
            entry: sys_entry.clone(),
            header: test_chain_header(),
        };

        let new_dht_store = reduce_hold_entry(
            Arc::clone(&context),
            &store.dht(),
            &ActionWrapper::new(Action::Hold(entry_wh)),
        )
        .expect("there should be a new store for committing a sys entry");

        assert_eq!(
            Some(sys_entry.clone()),
            (*storage.read().unwrap())
                .fetch(&sys_entry.address())
                .expect("could not fetch from cas")
                .map(|s| Entry::try_from_content(&s).unwrap())
        );

        let new_storage = &new_dht_store.content_storage().clone();
        assert_eq!(
            Some(sys_entry.clone()),
            (*new_storage.read().unwrap())
                .fetch(&sys_entry.address())
                .expect("could not fetch from cas")
                .map(|s| Entry::try_from_content(&s).unwrap())
        );
    }

    #[test]
    fn can_add_links() {
        let context = test_context("bob", None);
        let store = test_store(context.clone());
        let entry = test_entry();

        let locked_state = Arc::new(RwLock::new(store));

        let mut context = (*context).clone();
        context.set_state(locked_state.clone());
        let storage = context.dht_storage.clone();
        let _ = (storage.write().unwrap()).add(&entry);
        let context = Arc::new(context);

        let link = Link::new(&entry.address(), &entry.address(), "test-tag", "test-type");
        let action = ActionWrapper::new(Action::AddLink(link.clone()));

        let new_dht_store: DhtStore;
        {
            let state = locked_state.read().unwrap();

            new_dht_store = (*reduce(Arc::clone(&context), state.dht(), &action)).clone();
        }
        let storage = new_dht_store.meta_storage();
        let fetched = storage.read().unwrap().fetch_eavi(&EaviQuery::new(
            Some(entry.address()).into(),
            None.into(),
            None.into(),
            IndexFilter::LatestByAttribute,
        ));

        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 1);
        let eav = hash_set.iter().nth(0).unwrap();
        assert_eq!(eav.entity(), *link.base());
        assert_eq!(eav.value(), *link.target());
        assert_eq!(eav.attribute(), Attribute::LinkTag(link.tag().to_owned()));
    }

    #[test]
    fn can_remove_links() {
        let context = test_context("bob", None);
        let store = test_store(context.clone());
        let entry = test_entry();

        let locked_state = Arc::new(RwLock::new(store));

        let mut context = (*context).clone();
        context.set_state(locked_state.clone());
        let storage = context.dht_storage.clone();
        let _ = (storage.write().unwrap()).add(&entry);
        let context = Arc::new(context);

        let link = Link::new(&entry.address(), &entry.address(), "test-tag", "test-type");
        let mut action = ActionWrapper::new(Action::AddLink(link.clone()));

        let new_dht_store: DhtStore;
        {
            let state = locked_state.read().unwrap();

            new_dht_store = (*reduce(Arc::clone(&context), state.dht(), &action)).clone();
        }
        action = ActionWrapper::new(Action::RemoveLink(link.clone()));

        let _ = new_dht_store.meta_storage();

        let new_dht_store: DhtStore;
        {
            let state = locked_state.read().unwrap();

            new_dht_store = (*reduce(Arc::clone(&context), state.dht(), &action)).clone();
        }
        let storage = new_dht_store.meta_storage();
        let fetched = storage.read().unwrap().fetch_eavi(&EaviQuery::new(
            Some(entry.address()).into(),
            EavFilter::predicate(|a| match a {
                Attribute::LinkTag(_) | Attribute::RemovedLink(_) => true,
                _ => false,
            }),
            None.into(),
            IndexFilter::LatestByAttribute,
        ));

        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 1);
        let eav = hash_set.iter().nth(0).unwrap();
        assert_eq!(eav.entity(), *link.base());
        assert_eq!(eav.value(), *link.target());
        assert_eq!(
            eav.attribute(),
            Attribute::RemovedLink(link.tag().to_string())
        );
    }

    #[test]
    fn does_not_add_link_for_missing_base() {
        let context = test_context("bob", None);
        let store = test_store(context.clone());
        let entry = test_entry();

        let locked_state = Arc::new(RwLock::new(store));

        let mut context = (*context).clone();
        context.set_state(locked_state.clone());
        let context = Arc::new(context);

        let link = Link::new(&entry.address(), &entry.address(), "test-tag", "test-type");
        let action = ActionWrapper::new(Action::AddLink(link.clone()));

        let new_dht_store: DhtStore;
        {
            let state = locked_state.read().unwrap();

            new_dht_store = (*reduce(Arc::clone(&context), state.dht(), &action)).clone();
        }
        let storage = new_dht_store.meta_storage();
        let fetched = storage.read().unwrap().fetch_eavi(&EaviQuery::new(
            Some(entry.address()).into(),
            None.into(),
            None.into(),
            IndexFilter::LatestByAttribute,
        ));

        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 0);

        let result = new_dht_store.actions().get(&action).unwrap();

        assert!(result.is_err());
    }

    #[test]
    pub fn reduce_hold_test() {
        let context = test_context("bill", None);
        let store = test_store(context.clone());

        let entry = test_entry();
        let entry_wh = EntryWithHeader {
            entry: entry.clone(),
            header: test_chain_header(),
        };
        let action_wrapper = ActionWrapper::new(Action::Hold(entry_wh.clone()));

        store.reduce(context.clone(), action_wrapper);

        let cas = context.dht_storage.read().unwrap();

        let maybe_json = cas.fetch(&entry.address()).unwrap();
        let result_entry = match maybe_json {
            Some(content) => Entry::try_from(content).unwrap(),
            None => panic!("Could not find received entry in CAS"),
        };

        assert_eq!(&entry, &result_entry,);
    }

}
