//! all DHT reducers

use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::dht_store::DhtStore,
    nucleus::actions::get_entry::get_entry_rec,
};
use holochain_core_types::{
    cas::content::AddressableContent,
    crud_status::{create_crud_link_eav, create_crud_status_eav, CrudStatus, STATUS_NAME},
    eav::EntityAttributeValue,
    entry::{Entry, SerializedEntry},
    error::HolochainError,
};
use holochain_wasm_utils::api_serialization::get_entry::{
    GetEntryOptions, GetEntryResult, StatusRequestKind,
};
use std::{collections::HashSet, convert::TryFrom, sync::Arc};

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
        Action::Commit(_) => Some(reduce_add_crud_meta),
        Action::Hold(_) => Some(reduce_hold_entry),
        Action::GetEntry(_) => Some(reduce_get_entry_from_network),
        Action::UpdateEntry(_) => Some(reduce_update_entry),
        Action::RemoveEntry(_) => Some(reduce_remove_entry),
        Action::AddLink(_) => Some(reduce_add_link),
        //Action::GetLinks(_) => Some(reduce_get_links),
        _ => None,
    }
}

//
pub(crate) fn reduce_hold_entry(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let action = action_wrapper.action();
    let entry = unwrap_to!(action => Action::Hold);

    // Add it to local storage
    let new_store = (*old_store).clone();
    let storage = &new_store.content_storage().clone();
    let res = (*storage.write().unwrap()).add(entry);
    if res.is_err() {
        // TODO #439 - Log the error. Once we have better logging.
        return None;
    }
    // Done
    Some(new_store)
}

//
pub(crate) fn reduce_add_crud_meta(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // Get Action input
    let action = action_wrapper.action();
    let (entry, _maybe_crud) = unwrap_to!(action => Action::Commit);
    // Add crud-status metadata to local storage
    let new_store = (*old_store).clone();
    let meta_storage = &new_store.meta_storage().clone();
    let status_eav = create_crud_status_eav(&entry.address(), CrudStatus::LIVE);
    let res = (*meta_storage.write().unwrap()).add_eav(&status_eav);
    if res.is_err() {
        // TODO #439 - Log the error. Once we have better logging.
        println!(
            "reduce_add_crud_meta: meta_storage write failed!: {:?}",
            res.err().unwrap()
        );
        return None;
    }
    // Done
    Some(new_store)
}

//
pub(crate) fn reduce_get_entry_from_network(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // Get Action's input data
    let action = action_wrapper.action();
    let address = unwrap_to!(action => Action::GetEntry);
    let storage = &old_store.content_storage().clone();
    // pre-condition check: Look in local storage if it already has it.
    if (*storage.read().unwrap()).contains(address).unwrap() {
        // TODO #439 - Log a warning saying this should not happen. Once we have better logging.
        return None;
    }
    // Retrieve it from the network...
    old_store
        .network()
        .clone()
        .get(address)
        .and_then(|content| {
            let entry =
                Entry::try_from_content(&content).expect("could not load entry from content");
            let new_store = (*old_store).clone();

            // ...and add it to the local storage
            let storage = &new_store.content_storage().clone();
            let res = (*storage.write().unwrap()).add(&entry);
            match res {
                Err(_) => None,
                Ok(()) => Some(new_store),
            }
        })
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
        return Some(new_store);
    }

    let eav =
        EntityAttributeValue::new(link.base(), &format!("link__{}", link.tag()), link.target());

    let storage = new_store.meta_storage();
    let result = storage.write().unwrap().add_eav(&eav);
    new_store
        .actions_mut()
        .insert(action_wrapper.clone(), result.map(|_| link.base().clone()));
    Some(new_store)
}

//
pub(crate) fn reduce_update_entry(
    context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // Setup
    let action = action_wrapper.action();
    let (old_address, new_address) = unwrap_to!(action => Action::UpdateEntry);
    let mut new_store = (*old_store).clone();
    let content_storage = &old_store.content_storage().clone();
    // pre-condition: Must already have old_entry in local content_storage
    if !(*content_storage.read().unwrap())
        .contains(&old_address)
        .unwrap()
    {
        new_store.actions_mut().insert(
            action_wrapper.clone(),
            Err(HolochainError::ErrorGeneric(String::from(
                "old_entry is not present in DHT's CAS",
            ))),
        );
        return Some(new_store);
    }
    //  pre-condition: Must already have new_entry in local content_storage
    if !(*content_storage.read().unwrap())
        .contains(&new_address)
        .unwrap()
    {
        new_store.actions_mut().insert(
            action_wrapper.clone(),
            Err(HolochainError::ErrorGeneric(String::from(
                "new_entry is not present in DHT's CAS",
            ))),
        );
        return Some(new_store);
    }
    // pre-condition: old_entry's latest version must have LIVE crud-status
    // get latest entry
    let mut entry_result = GetEntryResult::new();
    let res = get_entry_rec(
        &context,
        &mut entry_result,
        old_address.clone(),
        GetEntryOptions::new(StatusRequestKind::Latest),
    );
    if let Err(err) = res {
        new_store
            .actions_mut()
            .insert(action_wrapper.clone(), Err(err));
        return Some(new_store);
    }
    let latest_old_address = entry_result.addresses.iter().last().unwrap();
    // verify its crud-status
    if entry_result.crud_status.iter().last().unwrap() != &CrudStatus::LIVE {
        new_store.actions_mut().insert(
            action_wrapper.clone(),
            Err(HolochainError::ErrorGeneric(String::from(
                "old_entry latest version does not have LIVE crud-status",
            ))),
        );
        return Some(new_store);
    }
    // pre-condition: latest entry must not already have a crud-link
    let maybe_crud_link = entry_result.crud_links.get(latest_old_address);
    if maybe_crud_link.is_some() {
        new_store.actions_mut().insert(
            action_wrapper.clone(),
            Err(HolochainError::ErrorGeneric(String::from(
                "attempted to add a second crud-link to an entry",
            ))),
        );
        return Some(new_store);
    }

    // Update crud-status
    let meta_storage = &new_store.meta_storage().clone();
    let new_status_eav = create_crud_status_eav(latest_old_address, CrudStatus::MODIFIED);
    let res = (*meta_storage.write().unwrap()).add_eav(&new_status_eav);
    if let Err(err) = res {
        new_store
            .actions_mut()
            .insert(action_wrapper.clone(), Err(err));
        return Some(new_store);
    }
    // Update crud-link
    let crud_link_eav = create_crud_link_eav(latest_old_address, new_address);
    let res = (*meta_storage.write().unwrap()).add_eav(&crud_link_eav);
    if let Err(err) = res {
        new_store
            .actions_mut()
            .insert(action_wrapper.clone(), Err(err));
        return Some(new_store);
    }
    // Done
    new_store
        .actions_mut()
        .insert(action_wrapper.clone(), res.map(|_| new_address.clone()));
    Some(new_store)
}

//
pub(crate) fn reduce_remove_entry(
    context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // Setup
    let action = action_wrapper.action();
    let (deleted_address, deletion_address) = unwrap_to!(action => Action::RemoveEntry);
    let mut new_store = (*old_store).clone();
    // Get latest entry
    let mut entry_result = GetEntryResult::new();
    let res = get_entry_rec(
        &context,
        &mut entry_result,
        deleted_address.clone(),
        GetEntryOptions::new(StatusRequestKind::Latest),
    );
    if let Err(err) = res {
        new_store
            .actions_mut()
            .insert(action_wrapper.clone(), Err(err));
        return Some(new_store);
    }

    let latest_deleted_address = entry_result.addresses.iter().last().unwrap();
    // pre-condition: Must already have entry in local content_storage
    let content_storage = &old_store.content_storage().clone();
    let maybe_entry = content_storage
        .read()
        .unwrap()
        .fetch(latest_deleted_address)
        .unwrap();
    if maybe_entry.is_none() {
        new_store.actions_mut().insert(
            action_wrapper.clone(),
            Err(HolochainError::ErrorGeneric(String::from(
                "trying to remove a missing entry",
            ))),
        );
        return Some(new_store);
    }
    let ser_entry = SerializedEntry::try_from(maybe_entry.unwrap()).unwrap();
    let entry = Entry::from(ser_entry);
    // pre-condition: entry_type must not by sys type, since they cannot be deleted
    if entry.entry_type().to_owned().is_sys() {
        new_store.actions_mut().insert(
            action_wrapper.clone(),
            Err(HolochainError::ErrorGeneric(String::from(
                "trying to remove a system entry type",
            ))),
        );
        return Some(new_store);
    }
    // pre-condition: Current status must be LIVE
    // get current status
    let meta_storage = &old_store.meta_storage().clone();
    let maybe_status_eav = meta_storage.read().unwrap().fetch_eav(
        Some(latest_deleted_address.clone()),
        Some(STATUS_NAME.to_string()),
        None,
    );
    if let Err(err) = maybe_status_eav {
        new_store
            .actions_mut()
            .insert(action_wrapper.clone(), Err(err));
        return Some(new_store);
    }
    let status_eavs = maybe_status_eav.unwrap();
    assert!(!status_eavs.is_empty(), "Entry should have a Status");
    // TODO waiting for update/remove_eav() assert!(status_eavs.len() <= 1);
    // For now checks if crud-status other than LIVE are present
    let status_eavs = status_eavs
        .iter()
        .filter(|e| CrudStatus::from(String::from(e.value())) != CrudStatus::LIVE)
        .collect::<HashSet<&EntityAttributeValue>>();
    if !status_eavs.is_empty() {
        new_store.actions_mut().insert(
            action_wrapper.clone(),
            Err(HolochainError::ErrorGeneric(String::from(
                "entry_status != CrudStatus::LIVE",
            ))),
        );
        return Some(new_store);
    }

    // Update crud-status
    let new_status_eav = create_crud_status_eav(latest_deleted_address, CrudStatus::DELETED);
    let meta_storage = &new_store.meta_storage().clone();
    let res = (*meta_storage.write().unwrap()).add_eav(&new_status_eav);
    if let Err(err) = res {
        new_store
            .actions_mut()
            .insert(action_wrapper.clone(), Err(err));
        return Some(new_store);
    }
    // Update crud-link
    let crud_link_eav = create_crud_link_eav(latest_deleted_address, deletion_address);
    let res = (*meta_storage.write().unwrap()).add_eav(&crud_link_eav);
    if let Err(err) = res {
        new_store
            .actions_mut()
            .insert(action_wrapper.clone(), Err(err));
        return Some(new_store);
    }
    // Done
    new_store.actions_mut().insert(
        action_wrapper.clone(),
        res.map(|_| latest_deleted_address.clone()),
    );
    Some(new_store)
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
        state::test_store,
    };
    use holochain_core_types::{
        cas::content::AddressableContent,
        entry::{test_entry, test_sys_entry, Entry},
        link::Link,
    };
    use std::{
        convert::TryFrom,
        sync::{Arc, RwLock},
    };

    #[test]
    fn reduce_hold_entry_test() {
        let context = test_context("bob");
        let store = test_store(context.clone());

        // test_entry is not sys so should do nothing
        let storage = &store.dht().content_storage().clone();

        let sys_entry = test_sys_entry();

        let new_dht_store = reduce_hold_entry(
            Arc::clone(&context),
            &store.dht(),
            &ActionWrapper::new(Action::Hold(sys_entry.clone())),
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
        let context = test_context("bob");
        let store = test_store(context.clone());
        let entry = test_entry();

        let locked_state = Arc::new(RwLock::new(store));

        let mut context = (*context).clone();
        context.set_state(locked_state.clone());
        let storage = context.file_storage.clone();
        let _ = (storage.write().unwrap()).add(&entry);
        let context = Arc::new(context);

        let link = Link::new(&entry.address(), &entry.address(), "test-tag");
        let action = ActionWrapper::new(Action::AddLink(link.clone()));

        let new_dht_store: DhtStore;
        {
            let state = locked_state.read().unwrap();

            new_dht_store = (*reduce(Arc::clone(&context), state.dht(), &action)).clone();
        }
        let storage = new_dht_store.meta_storage();
        let fetched = storage
            .read()
            .unwrap()
            .fetch_eav(Some(entry.address()), None, None);

        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 1);
        let eav = hash_set.iter().nth(0).unwrap();
        assert_eq!(eav.entity(), *link.base());
        assert_eq!(eav.value(), *link.target());
        assert_eq!(eav.attribute(), format!("link__{}", link.tag()));
    }

    #[test]
    fn does_not_add_link_for_missing_base() {
        let context = test_context("bob");
        let store = test_store(context.clone());
        let entry = test_entry();

        let locked_state = Arc::new(RwLock::new(store));

        let mut context = (*context).clone();
        context.set_state(locked_state.clone());
        let context = Arc::new(context);

        let link = Link::new(&entry.address(), &entry.address(), "test-tag");
        let action = ActionWrapper::new(Action::AddLink(link.clone()));

        let new_dht_store: DhtStore;
        {
            let state = locked_state.read().unwrap();

            new_dht_store = (*reduce(Arc::clone(&context), state.dht(), &action)).clone();
        }
        let storage = new_dht_store.meta_storage();
        let fetched = storage
            .read()
            .unwrap()
            .fetch_eav(Some(entry.address()), None, None);

        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 0);

        let result = new_dht_store.actions().get(&action).unwrap();

        assert!(result.is_err());
    }

    #[test]
    pub fn reduce_hold_test() {
        let context = test_context("bill");
        let store = test_store(context.clone());

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::Hold(entry.clone()));

        store.reduce(context.clone(), action_wrapper);

        let cas = context.file_storage.read().unwrap();

        let maybe_json = cas.fetch(&entry.address()).unwrap();
        let result_entry = match maybe_json {
            Some(content) => Entry::try_from(content).unwrap(),
            None => panic!("Could not find received entry in CAS"),
        };

        assert_eq!(&entry, &result_entry,);
    }

}
