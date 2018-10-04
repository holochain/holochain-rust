//! all DHT reducers

use action::Action;
use std::sync::Arc;
use action::ActionWrapper;
use context::Context;
use dht::dht_store::DhtStore;
use cas::storage::ContentAddressableStorage;
use cas::eav::EntityAttributeValueStorage;

// A function that might return a mutated DhtStore
type DhtReducer<CAS: ContentAddressableStorage, EAVS: EntityAttributeValueStorage> =
fn(Arc<Context>, &DhtStore<CAS, EAVS>, &ActionWrapper) -> Option<DhtStore<CAS, EAVS>>;

/// DHT state-slice Reduce entry point.
/// Note: Can't block when dispatching action here because we are inside the reduce's mutex
pub fn reduce<CAS: ContentAddressableStorage, EAVS: EntityAttributeValueStorage>(
    context: Arc<Context>,
    old_store: Arc<DhtStore<CAS, EAVS>>,
    action_wrapper: &ActionWrapper,
) -> Arc<DhtStore<CAS, EAVS>> {
    // Get reducer
    let maybe_reducer = resolve_reducer(action_wrapper);
    if maybe_reducer.is_none() {
        return old_store;
    }
    let reducer = maybe_reducer.unwrap();
    // Reduce
    let maybe_new_store = reducer(
        context,
        &old_store,
        &action_wrapper,
    );
    match maybe_new_store {
        None => old_store,
        Some(new_store) => Arc::new(new_store),
    }
}

/// Maps incoming action to the correct reducer
fn resolve_reducer<CAS: ContentAddressableStorage, EAVS: EntityAttributeValueStorage>(action_wrapper: &ActionWrapper)
    -> Option<DhtReducer<CAS, EAVS>> {
    match action_wrapper.action() {
        Action::Commit(_)   => Some(reduce_commit_entry),
        Action::GetEntry(_) => Some(reduce_get_entry),
        Action::AddLink(_)  => Some(reduce_add_link),
        Action::GetLinks(_) => Some(reduce_get_links),
        _ => None,
    }
}

//
pub(crate) fn reduce_commit_entry<CAS: ContentAddressableStorage, EAVS: EntityAttributeValueStorage>(
    _context: Arc<Context>,
    old_store: &DhtStore<CAS, EAVS>,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore<CAS, EAVS>> {
    let action = action_wrapper.action();
    let entry = unwrap_to!(action => Action::GetEntry);

    // Look in local storage if it already has it
    if old_store.content_storage().contains(entry.key()).unwrap() {
        // Maybe panic as this should never happen?
        return None;
    }
    // Otherwise add it local storage...
    let mut new_store = (*old_store).clone();
    new_store.content_storage().add(entry);
    // ...and publish to the network
    new_store.network().publish(entry);
    Some(new_store)
}

//
pub(crate) fn reduce_get_entry<CAS: ContentAddressableStorage, EAVS: EntityAttributeValueStorage>(
    _context: Arc<Context>,
    old_store: &DhtStore<CAS, EAVS>,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore<CAS, EAVS>> {
    // Get Action's input data
    let action = action_wrapper.action();
    let hash = unwrap_to!(action => Action::GetEntry);

    // Look in local storage if it already has it
    if old_store.data_storage().contains(hash).unwrap() {
        return None;
    }
    // Otherwise retrieve it from the network...
    let mut new_store = (*old_store).clone();
    let content = old_store.network().get(hash);
    // ...and add it to the local storage
    new_store.data_storage().add(content);
    Some(new_store)
}

//
pub(crate) fn reduce_add_link<CAS: ContentAddressableStorage, EAVS: EntityAttributeValueStorage>(
    _context: Arc<Context>,
    old_store: &DhtStore<CAS, EAVS>,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore<CAS, EAVS>> {
    let action = action_wrapper.action();
    let link = unwrap_to!(action => Action::AddLink);

    // Look in local storage if it already has it
    if old_store.data_storage().contains(&link.key()).unwrap() {
        // TODO Maybe panic as this should never happen?
        return None;
    }
    // Otherwise add it to the local storage...
    let mut new_store = (*old_store).clone();
    // FIXME convert link to meta here
    new_store.add_link(link);
    let link_meta;
    // ... and publish to the network
    new_store.network().publish_meta(link_meta);
    Some(new_store)
}

//
pub(crate) fn reduce_get_links<CAS: ContentAddressableStorage, EAVS: EntityAttributeValueStorage>(
    _context: Arc<Context>,
    old_store: &DhtStore<CAS, EAVS>,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore<CAS, EAVS>> {
    // Get Action's input data
    let action = action_wrapper.action();
    let get_links_args = unwrap_to!(action => Action::GetLinks);

    // retrieve it from the network?
    // FIXME
    let mut new_store = (*old_store).clone();
    // ...
    Some(new_store)
}
