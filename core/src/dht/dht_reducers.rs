//! all DHT reducers

use action::{Action, ActionWrapper};
use cas::{content::AddressableContent, storage::ContentAddressableStorage};
use context::Context;
use dht::dht_store::DhtStore;
use eav::EntityAttributeValueStorage;
use hash_table::entry::Entry;
use std::sync::Arc;

// A function that might return a mutated DhtStore
type DhtReducer<CAS, EAVS> =
    fn(Arc<Context>, &DhtStore<CAS, EAVS>, &ActionWrapper) -> Option<DhtStore<CAS, EAVS>>;

/// DHT state-slice Reduce entry point.
/// Note: Can't block when dispatching action here because we are inside the reduce's mutex
pub fn reduce<CAS, EAVS>(
    context: Arc<Context>,
    old_store: Arc<DhtStore<CAS, EAVS>>,
    action_wrapper: &ActionWrapper,
) -> Arc<DhtStore<CAS, EAVS>>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    // Get reducer
    let maybe_reducer = resolve_reducer(action_wrapper);
    if maybe_reducer.is_none() {
        return old_store;
    }
    let reducer = maybe_reducer.unwrap();
    // Reduce
    let maybe_new_store = reducer(context, &old_store, &action_wrapper);
    match maybe_new_store {
        None => old_store,
        Some(new_store) => Arc::new(new_store),
    }
}

/// Maps incoming action to the correct reducer
fn resolve_reducer<CAS, EAVS>(action_wrapper: &ActionWrapper) -> Option<DhtReducer<CAS, EAVS>>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    match action_wrapper.action() {
        Action::Commit(_, _) => Some(reduce_commit_entry),
        Action::GetEntry(_) => Some(reduce_get_entry_from_network),
        Action::AddLink(_) => Some(reduce_add_link),
        Action::GetLinks(_) => Some(reduce_get_links),
        _ => None,
    }
}

//
pub(crate) fn reduce_commit_entry<CAS, EAVS>(
    _context: Arc<Context>,
    old_store: &DhtStore<CAS, EAVS>,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore<CAS, EAVS>>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    let action = action_wrapper.action();
    let (entry_type, entry) = match action {
        Action::Commit(entry_type, entry) => (entry_type, entry),
        _ => unreachable!(),
    };
    // Look in local storage if it already has it
    if old_store
        .content_storage()
        .contains(&entry.address())
        .unwrap()
    {
        // TODO #439 - Log a warning saying this should not happen. Once we have better logging.
        return None;
    }
    // Otherwise add it local storage...
    let mut new_store = (*old_store).clone();
    let res = new_store.content_storage_mut().add(entry);
    if res.is_err() {
        // TODO #439 - Log the error. Once we have better logging.
        return None;
    }
    // ...and publish to the network
    // TODO #440 - Must check if entry is "publishable" (i.e. public)
    new_store.network_mut().publish(entry);
    Some(new_store)
}

//
pub(crate) fn reduce_get_entry_from_network<CAS, EAVS>(
    _context: Arc<Context>,
    old_store: &DhtStore<CAS, EAVS>,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore<CAS, EAVS>>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    // Get Action's input data
    let action = action_wrapper.action();
    let address = unwrap_to!(action => Action::GetEntry);
    // pre-condition check: Look in local storage if it already has it.
    if old_store.content_storage().contains(address).unwrap() {
        // TODO #439 - Log a warning saying this should not happen. Once we have better logging.
        return None;
    }
    // Retrieve it from the network...
    let entry = Entry::from_content(&old_store.network().clone().get(address));
    let mut new_store = (*old_store).clone();
    // ...and add it to the local storage
    let res = new_store.content_storage_mut().add(&entry);
    match res {
        Err(_) => None,
        Ok(()) => Some(new_store),
    }
}

//
pub(crate) fn reduce_add_link<CAS, EAVS>(
    _context: Arc<Context>,
    _old_store: &DhtStore<CAS, EAVS>,
    _action_wrapper: &ActionWrapper,
) -> Option<DhtStore<CAS, EAVS>>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    // FIXME
    None
}

//
pub(crate) fn reduce_get_links<CAS, EAVS>(
    _context: Arc<Context>,
    _old_store: &DhtStore<CAS, EAVS>,
    _action_wrapper: &ActionWrapper,
) -> Option<DhtStore<CAS, EAVS>>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    // FIXME
    None
}
