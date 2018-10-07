//! all DHT reducers

use action::{Action, ActionWrapper};
use holochain_core_types::{
    cas::{content::AddressableContent, storage::ContentAddressableStorage},
    eav::EntityAttributeValueStorage,
    entry::Entry, entry_type::EntryType
};
use context::Context;
use dht::dht_store::DhtStore;
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
pub(crate) fn commit_sys_entry<CAS, EAVS>(
    old_store: &DhtStore<CAS, EAVS>,
    entry_type: &EntryType,
    entry: &Entry,
) -> Option<DhtStore<CAS, EAVS>>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    // system entry type must be publishable
    if !entry_type.clone().can_publish() {
        return None;
    }
    // Add it local storage
    let mut new_store = (*old_store).clone();
    let res = new_store.content_storage_mut().add(entry);
    if res.is_err() {
        // TODO #439 - Log the error. Once we have better logging.
        return None;
    }
    // Note: System entry types are not published to the network
    Some(new_store)
}

//
pub(crate) fn commit_app_entry<CAS, EAVS>(
    context: Arc<Context>,
    old_store: &DhtStore<CAS, EAVS>,
    entry_type: &EntryType,
    entry: &Entry,
) -> Option<DhtStore<CAS, EAVS>>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    // pre-condition: if app entry_type must be valid
    // get entry_type definition
    let dna = context
        .state()
        .expect("context must have a State.")
        .nucleus()
        .dna()
        .expect("context.state must hold DNA in order to commit an app entry.");
    let maybe_def = dna.get_entry_type_def(&entry_type.to_string());
    if maybe_def.is_none() {
        // TODO #439 - Log the error. Once we have better logging.
        return None;
    }
    let entry_type_def = maybe_def.unwrap();

    // app entry type must be publishable
    if !entry_type_def.sharing.clone().can_publish() {
        return None;
    }

    // Add it to local storage...
    let mut new_store = (*old_store).clone();
    let res = new_store.content_storage_mut().add(entry);
    if res.is_err() {
        // TODO #439 - Log the error. Once we have better logging.
        return None;
    }
    // ...and publish to the network if its not private
    new_store.network_mut().publish(entry);
    // Done
    Some(new_store)
}

//
pub(crate) fn reduce_commit_entry<CAS, EAVS>(
    context: Arc<Context>,
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

    // pre-condition: Must not already have entry in local storage
    if old_store
        .content_storage()
        .contains(&entry.address())
        .unwrap()
    {
        // TODO #439 - Log a warning saying this should not happen. Once we have better logging.
        return None;
    }

    // Handle sys entries and app entries differently
    if entry_type.clone().is_sys() {
        return commit_sys_entry(old_store, entry_type, entry);
    }
    return commit_app_entry(context, old_store, entry_type, entry);
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
