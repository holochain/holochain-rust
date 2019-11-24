//! all DHT reducers

use crate::{
    action::{Action, ActionWrapper},
    dht::dht_store::DhtStore,
    network::entry_with_header::EntryWithHeader,
};
use std::sync::Arc;

use super::dht_inner_reducers::{
    reduce_add_remove_link_inner, reduce_remove_entry_inner, reduce_store_entry_inner,
    reduce_update_entry_inner, LinkModification,
};

use holochain_core_types::entry::Entry;
use holochain_persistence_api::cas::content::AddressableContent;

// A function that might return a mutated DhtStore
type DhtReducer = fn(&DhtStore, &ActionWrapper) -> Option<DhtStore>;

/// DHT state-slice Reduce entry point.
/// Note: Can't block when dispatching action here because we are inside the reduce's mutex
pub fn reduce(old_store: Arc<DhtStore>, action_wrapper: &ActionWrapper) -> Arc<DhtStore> {
    // Get reducer
    let reducer = match resolve_reducer(action_wrapper) {
        Some(reducer) => reducer,
        None => {
            return old_store;
        }
    };
    // Reduce
    match reducer(&old_store.clone(), &action_wrapper) {
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
        Action::QueueHoldingWorkflow(_) => Some(reduce_queue_holding_workflow),
        Action::RemoveQueuedHoldingWorkflow(_) => Some(reduce_remove_queued_holding_workflow),
        _ => None,
    }
}

pub(crate) fn reduce_commit_entry(
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let (entry, _, _) = unwrap_to!(action_wrapper.action() => Action::Commit);
    let mut new_store = (*old_store).clone();
    match reduce_store_entry_inner(&mut new_store, entry) {
        Ok(()) => Some(new_store),
        Err(e) => {
            println!("{}", e);
            None
        }
    }
}

pub(crate) fn reduce_hold_entry(
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let EntryWithHeader { entry, header } = unwrap_to!(action_wrapper.action() => Action::Hold);
    let mut new_store = (*old_store).clone();
    match reduce_store_entry_inner(&mut new_store, &entry) {
        Ok(()) => {
            new_store.add_header_for_entry(&entry, &header).ok()?;
            Some(new_store)
        }
        Err(e) => {
            println!("{}", e);
            None
        }
    }
}

pub(crate) fn reduce_add_link(
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let link_data = unwrap_to!(action_wrapper.action() => Action::AddLink);
    let mut new_store = (*old_store).clone();
    let entry = Entry::LinkAdd(link_data.clone());
    let res = reduce_add_remove_link_inner(
        &mut new_store,
        link_data.link(),
        &entry.address(),
        LinkModification::Add,
    );
    new_store.actions_mut().insert(action_wrapper.clone(), res);
    Some(new_store)
}

pub(crate) fn reduce_remove_link(
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let entry = unwrap_to!(action_wrapper.action() => Action::RemoveLink);
    let (link_data, links_to_remove) = unwrap_to!(entry => Entry::LinkRemove);
    let new_store = (*old_store).clone();
    let store = links_to_remove
        .iter()
        .fold(new_store, |mut store, link_addresses| {
            let res = reduce_add_remove_link_inner(
                &mut store,
                link_data.link(),
                link_addresses,
                LinkModification::Remove,
            );
            store.actions_mut().insert(action_wrapper.clone(), res);
            store.clone()
        });

    Some(store)
}

pub(crate) fn reduce_update_entry(
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
    _old_store: &DhtStore,
    _action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // FIXME
    None
}

#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
pub fn reduce_queue_holding_workflow(
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let action = action_wrapper.action();
    let pending = unwrap_to!(action => Action::QueueHoldingWorkflow);
    let mut new_store = (*old_store).clone();
    new_store
        .queued_holding_workflows
        .push_back(pending.clone());
    Some(new_store)
}

#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
pub fn reduce_remove_queued_holding_workflow(
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let action = action_wrapper.action();
    let pending = unwrap_to!(action => Action::RemoveQueuedHoldingWorkflow);
    let mut new_store = (*old_store).clone();
    if let Some((front, _)) = new_store.queued_holding_workflows.front() {
        if front == pending {
            let _ = new_store.queued_holding_workflows.pop_front();
        } else {
            // The first item in the queue could be a delayed one which will result
            // in the holding thread seeing another item as the next one.
            // The holding thread will still try to pop that next item, so we need
            // this else case where we just remove an item from some position inside the queue:
            new_store
                .queued_holding_workflows
                .retain(|(item, _)| item != pending);
        }
    } else {
        error!("Got Action::PopNextHoldingWorkflow on an empty holding queue!");
    }

    Some(new_store)
}

#[cfg(test)]
pub mod tests {

    use crate::{
        action::{Action, ActionWrapper},
        content_store::GetContent,
        dht::{
            dht_reducers::{
                reduce, reduce_hold_entry, reduce_queue_holding_workflow,
                reduce_remove_queued_holding_workflow,
            },
            dht_store::{create_get_links_eavi_query, DhtStore},
            pending_validations::{PendingValidation, PendingValidationStruct, ValidatingWorkflow},
        },
        instance::tests::test_context,
        network::entry_with_header::test_entry_with_header,
        state::test_store,
    };
    use bitflags::_core::time::Duration;
    use holochain_core_types::{
        agent::{test_agent_id, test_agent_id_with_name},
        chain_header::test_chain_header,
        eav::Attribute,
        entry::{test_entry, Entry},
        link::{link_data::LinkData, Link, LinkActionKind},
    };
    use holochain_persistence_api::cas::content::AddressableContent;
    use std::{sync::Arc, time::SystemTime};

    #[test]
    fn reduce_hold_entry_test() {
        let context = test_context("bob", None);
        let store = test_store(context);

        // test_entry is not sys so should do nothing
        let entry_wh = test_entry_with_header();
        let entry = entry_wh.entry.clone();

        let new_dht_store =
            reduce_hold_entry(&store.dht(), &ActionWrapper::new(Action::Hold(entry_wh)))
                .expect("there should be a new store for committing a sys entry");

        // the old store does not have the entry in its holding list
        assert_eq!(None, store.dht().get(&entry.address()).unwrap());

        assert_eq!(
            Some(entry.clone()),
            new_dht_store
                .get(&entry.address())
                .expect("could not fetch from cas")
        );
    }

    #[test]
    fn can_add_links() {
        let context = test_context("bob", None);
        let store = test_store(context.clone());
        let entry_wh = test_entry_with_header();
        let entry = entry_wh.entry.clone();

        let store = reduce_hold_entry(&store.dht(), &ActionWrapper::new(Action::Hold(entry_wh)))
            .expect("there should be a new store for committing a sys entry");

        let test_link = String::from("test_link");
        let test_tag = String::from("test-tag");
        let link = Link::new(
            &entry.address(),
            &entry.address(),
            &test_link.clone(),
            &test_tag.clone(),
        );
        let link_data = LinkData::from_link(
            &link,
            LinkActionKind::ADD,
            test_chain_header(),
            test_agent_id(),
        );
        let action = ActionWrapper::new(Action::AddLink(link_data.clone()));
        let link_entry = Entry::LinkAdd(link_data.clone());

        let store = (*reduce(Arc::new(store), &action)).clone();

        let get_links_query = create_get_links_eavi_query(entry.address(), test_link, test_tag)
            .expect("supposed to create link query");
        let fetched = store.fetch_eavi(&get_links_query);
        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 1);
        let eav = hash_set.iter().nth(0).unwrap();
        assert_eq!(eav.entity(), *link.base());
        assert_eq!(eav.value(), link_entry.address());
        assert_eq!(
            eav.attribute(),
            Attribute::LinkTag(link.link_type().to_owned(), link.tag().to_owned())
        );
    }

    #[test]
    fn can_remove_links() {
        let context = test_context("bob", None);
        let store = test_store(context.clone());
        let entry_wh = test_entry_with_header();
        let entry = entry_wh.entry.clone();

        let store = reduce_hold_entry(&store.dht(), &ActionWrapper::new(Action::Hold(entry_wh)))
            .expect("there should be a new store for committing a sys entry");

        let test_link = String::from("test_link");
        let test_tag = String::from("test-tag");
        let link = Link::new(
            &entry.address(),
            &entry.address(),
            &test_link.clone(),
            &test_tag.clone(),
        );
        let link_data = LinkData::from_link(
            &link,
            LinkActionKind::ADD,
            test_chain_header(),
            test_agent_id(),
        );

        //add link to dht
        let entry_link_add = Entry::LinkAdd(link_data.clone());
        let action_link_add = ActionWrapper::new(Action::AddLink(link_data.clone()));
        let store = reduce(Arc::new(store), &action_link_add);

        let link_remove_data = LinkData::from_link(
            &link.clone(),
            LinkActionKind::REMOVE,
            test_chain_header(),
            test_agent_id(),
        );

        let entry_link_remove =
            Entry::LinkRemove((link_remove_data, vec![entry_link_add.clone().address()]));

        //remove added link from dht
        let action_link_remove = ActionWrapper::new(Action::RemoveLink(entry_link_remove.clone()));
        let store = reduce(store, &action_link_remove);

        //fetch from dht and when tombstone is found return tombstone
        let get_links_query =
            create_get_links_eavi_query(entry.address(), test_link.clone(), test_tag.clone())
                .expect("supposed to create link query");
        let fetched = store.fetch_eavi(&get_links_query);

        //fetch call should be okay and remove_link tombstone should be the one that should be returned
        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 1);
        let eav = hash_set.iter().nth(0).unwrap();
        assert_eq!(eav.entity(), *link.base());
        let link_entry = link.add_entry(test_chain_header(), test_agent_id());
        assert_eq!(eav.value(), link_entry.address());
        assert_eq!(
            eav.attribute(),
            Attribute::RemovedLink(link.link_type().to_string(), link.tag().to_string())
        );

        //add new link with same chain header
        let action_link_add = ActionWrapper::new(Action::AddLink(link_data));
        let store = reduce(store, &action_link_add);

        //fetch from dht after link with same chain header is added
        let get_links_query =
            create_get_links_eavi_query(entry.address(), test_link.clone(), test_tag.clone())
                .expect("supposed to create link query");
        let fetched = store.fetch_eavi(&get_links_query);

        //fetch call should be okay and remove_link tombstone should be the one that should be returned since tombstone is applied to target hashes that are the same
        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 1);
        let eav = hash_set.iter().nth(0).unwrap();
        assert_eq!(eav.entity(), *link.base());
        let link_entry = link.add_entry(test_chain_header(), test_agent_id());
        assert_eq!(eav.value(), link_entry.address());
        assert_eq!(
            eav.attribute(),
            Attribute::RemovedLink(link.link_type().to_string(), link.tag().to_string())
        );

        //add new link after tombstone has been added with different chain_header which will produce different hash
        let link_data = LinkData::from_link(
            &link.clone(),
            LinkActionKind::ADD,
            test_chain_header(),
            test_agent_id_with_name("new_agent"),
        );
        let entry_link_add = Entry::LinkAdd(link_data.clone());
        let action_link_add = ActionWrapper::new(Action::AddLink(link_data));
        let store = reduce(store, &action_link_add);

        //after new link has been added return from fetch and make sure tombstone and new link is added
        let get_links_query = create_get_links_eavi_query(entry.address(), test_link, test_tag)
            .expect("supposed to create link query");
        let fetched = store.fetch_eavi(&get_links_query);

        //two entries should be returned which is the new_link and the tombstone since the tombstone doesn't apply for the new link
        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 2);
        let eav = hash_set.iter().nth(1).unwrap();
        assert_eq!(eav.entity(), *link.base());
        let _link_entry = link.add_entry(test_chain_header(), test_agent_id());
        assert_eq!(eav.value(), entry_link_add.address());
        assert_eq!(
            eav.attribute(),
            Attribute::LinkTag(link.link_type().to_string(), link.tag().to_string())
        );
    }

    #[test]
    fn does_not_add_link_for_missing_base() {
        let context = test_context("bob", None);
        let store = test_store(context.clone());
        let entry = test_entry();
        let test_link = String::from("test-link-type");
        let test_tag = String::from("test-tag");
        let link = Link::new(
            &entry.address(),
            &entry.address(),
            &test_link.clone(),
            &test_tag.clone(),
        );

        let link_data = LinkData::from_link(
            &link.clone(),
            LinkActionKind::ADD,
            test_chain_header(),
            test_agent_id(),
        );
        let action = ActionWrapper::new(Action::AddLink(link_data));

        let new_dht_store = reduce(store.dht(), &action);

        let get_links_query = create_get_links_eavi_query(entry.address(), test_link, test_tag)
            .expect("supposed to create link query");
        let fetched = new_dht_store.fetch_eavi(&get_links_query);
        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 0);

        let result = new_dht_store.actions().get(&action).unwrap();

        assert!(result.is_err());
    }

    // TODO: Bring the old in-memory network up to speed and turn on this test again!
    #[cfg(feature = "broken-tests")]
    #[test]
    #[cfg(feature = "broken-tests")]
    pub fn reduce_hold_test() {
        let context = test_context("bill", None);
        let store = test_store(context.clone());

        let entry_wh = test_entry_with_header();
        let entry = entry_wh.entry.clone();

        let action_wrapper = ActionWrapper::new(Action::Hold(entry_wh.clone()));

        store.reduce(action_wrapper);

        let cas = context.dht_storage.read().unwrap();

        let maybe_json = cas.fetch(&entry.address()).unwrap();
        let result_entry = match maybe_json {
            Some(content) => Entry::try_from(content).unwrap(),
            None => panic!("Could not find received entry in CAS"),
        };

        assert_eq!(&entry, &result_entry,);
    }

    fn create_pending_validation(workflow: ValidatingWorkflow) -> PendingValidation {
        let entry_wh = test_entry_with_header();
        Arc::new(PendingValidationStruct::new(entry_wh, workflow))
    }

    #[test]
    pub fn test_holding_queue() {
        let context = test_context("test", None);
        let store = DhtStore::new(context.dht_storage.clone(), context.eav_storage.clone());
        assert_eq!(store.queued_holding_workflows().len(), 0);

        let hold = create_pending_validation(ValidatingWorkflow::HoldEntry);
        let action = ActionWrapper::new(Action::QueueHoldingWorkflow((
            hold.clone(),
            Some((SystemTime::now(), Duration::from_secs(10000))),
        )));
        let store = reduce_queue_holding_workflow(&store, &action).unwrap();

        assert_eq!(store.queued_holding_workflows().len(), 1);
        assert!(store.has_queued_holding_workflow(&hold));

        let hold_link = create_pending_validation(ValidatingWorkflow::HoldLink);
        let action = ActionWrapper::new(Action::QueueHoldingWorkflow((hold_link.clone(), None)));
        let store = reduce_queue_holding_workflow(&store, &action).unwrap();

        assert_eq!(store.queued_holding_workflows().len(), 2);
        assert!(store.has_queued_holding_workflow(&hold_link));

        let update = create_pending_validation(ValidatingWorkflow::UpdateEntry);
        let action = ActionWrapper::new(Action::QueueHoldingWorkflow((update.clone(), None)));
        let store = reduce_queue_holding_workflow(&store, &action).unwrap();

        assert_eq!(store.queued_holding_workflows().len(), 3);
        assert!(store.has_queued_holding_workflow(&update));

        let (next_pending, _) = store.next_queued_holding_workflow().unwrap();
        assert_eq!(hold_link, *next_pending);

        let action = ActionWrapper::new(Action::RemoveQueuedHoldingWorkflow(hold_link.clone()));
        let store = reduce_remove_queued_holding_workflow(&store, &action).unwrap();

        assert_eq!(store.queued_holding_workflows().len(), 2);
        assert!(!store.has_queued_holding_workflow(&hold_link));

        assert!(store.has_queued_holding_workflow(&hold));
        assert!(store.has_queued_holding_workflow(&update));

        let (next_pending, _) = store.next_queued_holding_workflow().unwrap();
        assert_eq!(update, *next_pending);

        let action = ActionWrapper::new(Action::RemoveQueuedHoldingWorkflow(hold.clone()));
        let store = reduce_remove_queued_holding_workflow(&store, &action).unwrap();

        assert_eq!(store.queued_holding_workflows().len(), 1);
        assert!(!store.has_queued_holding_workflow(&hold));
        assert!(store.has_queued_holding_workflow(&update));

        let (next_pending, _) = store.next_queued_holding_workflow().unwrap();
        assert_eq!(update, *next_pending);
    }
}
