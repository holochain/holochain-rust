//! all DHT reducers

use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::dht_store::DhtStore,
};
use holochain_core_types::{
    eav::EntityAttributeValue,
    error::HolochainError,
};
use std::sync::Arc;

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
        Action::Hold(_) => Some(reduce_hold_entry),
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

    // Add it to local storage...
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
        new_store.add_link_actions_mut().insert(
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
        .add_link_actions_mut()
        .insert(action_wrapper.clone(), result);
    Some(new_store)
}

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
        entry::{test_entry, test_sys_entry, Entry, SerializedEntry},
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

        let result = new_dht_store.add_link_actions().get(&action).unwrap();

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
            Some(content) => SerializedEntry::try_from(content).unwrap().deserialize(),
            None => panic!("Could not find received entry in CAS"),
        };

        assert_eq!(&entry, &result_entry,);
    }

}
