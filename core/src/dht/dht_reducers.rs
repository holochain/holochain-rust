//! all DHT reducers

use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::dht_store::DhtStore,
    network::entry_with_header::EntryWithHeader,
};
use std::sync::Arc;

use super::dht_inner_reducers::{
    reduce_add_remove_link_inner, reduce_remove_entry_inner, reduce_store_entry_inner,
    reduce_update_entry_inner, LinkModification,
};

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
    let reducer = match resolve_reducer(action_wrapper) {
        Some(reducer) => reducer,
        None => {
            return old_store;
        }
    };
    // Reduce
    match reducer(context, &old_store.clone(), &action_wrapper) {
        None => old_store,
        Some(new_store) => Arc::new(new_store),
    }
}

/// Maps incoming action to the correct reducer
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<DhtReducer> {
    match action_wrapper.action() {
        Action::Commit(_) => Some(reduce_commit_entry),
        Action::Hold(_) => Some(reduce_hold_entry),
        Action::UpdateEntry(_) => Some(reduce_update_entry),
        Action::RemoveEntry(_) => Some(reduce_remove_entry),
        Action::AddLink(_) => Some(reduce_add_link),
        Action::RemoveLink(_) => Some(reduce_remove_link),
        _ => None,
    }
}

pub(crate) fn reduce_commit_entry(
    context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let (entry, _, _) = unwrap_to!(action_wrapper.action() => Action::Commit);
    let mut new_store = (*old_store).clone();
    match reduce_store_entry_inner(&mut new_store, entry) {
        Ok(()) => Some(new_store),
        Err(e) => {
            context.log(e);
            None
        }
    }
}

pub(crate) fn reduce_hold_entry(
    context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let EntryWithHeader { entry, header } = unwrap_to!(action_wrapper.action() => Action::Hold);
    let mut new_store = (*old_store).clone();
    match reduce_store_entry_inner(&mut new_store, entry) {
        Ok(()) => {
            new_store.add_header_for_entry(&entry, &header).ok()?;
            Some(new_store)
        }
        Err(e) => {
            context.log(e);
            None
        }
    }
}

pub(crate) fn reduce_add_link(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let link = unwrap_to!(action_wrapper.action() => Action::AddLink);
    let mut new_store = (*old_store).clone();
    let res = reduce_add_remove_link_inner(&mut new_store, link, LinkModification::Add);
    new_store.actions_mut().insert(action_wrapper.clone(), res);
    Some(new_store)
}

pub(crate) fn reduce_remove_link(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let link = unwrap_to!(action_wrapper.action() => Action::RemoveLink);
    let mut new_store = (*old_store).clone();
    let res = reduce_add_remove_link_inner(&mut new_store, link, LinkModification::Remove);
    new_store.actions_mut().insert(action_wrapper.clone(), res);
    Some(new_store)
}

pub(crate) fn reduce_update_entry(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let (old_address, new_address) = unwrap_to!(action_wrapper.action() => Action::UpdateEntry);
    let mut new_store = (*old_store).clone();
    let res = reduce_update_entry_inner(&mut new_store, old_address, new_address);
    new_store.actions_mut().insert(action_wrapper.clone(), res);
    Some(new_store)
}

pub(crate) fn reduce_remove_entry(
    _context: Arc<Context>,
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let (deleted_address, deletion_address) =
        unwrap_to!(action_wrapper.action() => Action::RemoveEntry);
    let mut new_store = (*old_store).clone();
    let res = reduce_remove_entry_inner(&mut new_store, deleted_address, deletion_address);
    new_store.actions_mut().insert(action_wrapper.clone(), res);
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

        let link = Link::new(&entry.address(), &entry.address(), "test-tag");
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

        let link = Link::new(&entry.address(), &entry.address(), "test-tag");
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

        let link = Link::new(&entry.address(), &entry.address(), "test-tag");
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
