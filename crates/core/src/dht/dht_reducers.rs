//! all DHT reducers

use holochain_core_types::error::HolochainError;

use crate::{
    action::{Action, ActionWrapper},
    dht::{
        dht_store::DhtStore,
        pending_validations::{PendingValidationWithTimeout, ValidationTimeout},
    },
};
use std::sync::Arc;

use super::dht_inner_reducers::{
    reduce_add_remove_link_inner, reduce_remove_entry_inner, reduce_store_entry_inner,
    reduce_update_entry_inner, LinkModification,
};

use holochain_core_types::{entry::Entry, network::entry_aspect::EntryAspect};
use holochain_persistence_api::cas::content::AddressableContent;
use itertools::Itertools;
use std::collections::VecDeque;
// A function that might return a mutated DhtStore
type DhtReducer = fn(&DhtStore, &ActionWrapper) -> Option<DhtStore>;

/// DHT state-slice Reduce entry point.
/// Note: Can't block when dispatching action here because we are inside the reduce's mutex
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<DhtReducer> {
    match action_wrapper.action() {
        Action::Commit(_) => Some(reduce_commit_entry),
        Action::HoldAspect(_) => Some(reduce_hold_aspect),
        Action::QueueHoldingWorkflow(_) => Some(reduce_queue_holding_workflow),
        Action::RemoveQueuedHoldingWorkflow(_) => Some(reduce_remove_queued_holding_workflow),
        Action::Prune => Some(reduce_prune),
        _ => None,
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub(crate) fn reduce_hold_aspect(
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let (aspect, id) = unwrap_to!(action_wrapper.action() => Action::HoldAspect);
    let mut new_store = (*old_store).clone();

    // TODO: we think we don't need this but not 100%
    // new_store.actions_mut().insert(
    //     action_wrapper.clone(),
    //     Ok("TODO: nico, do we need this?".into()),
    // );
    let mut hold_result: Result<(), HolochainError> = Ok(());
    let mut maybe_store = match aspect {
        EntryAspect::Content(entry, header) => {
            match reduce_store_entry_inner(&mut new_store, &entry) {
                Ok(()) => {
                    new_store.add_header_for_entry(&entry, &header).ok()?;
                    Some(new_store)
                }
                Err(e) => {
                    let err = format!("EntryAspect::Content hold error: {}", e);
                    hold_result = Err(HolochainError::ErrorGeneric(err));
                    None
                }
            }
        }
        EntryAspect::LinkAdd(link_data, _header) => {
            let entry = Entry::LinkAdd(link_data.clone());
            match reduce_add_remove_link_inner(
                &mut new_store,
                &link_data,
                &entry.address(),
                LinkModification::Add,
            ) {
                Ok(_) => Some(new_store),
                Err(e) => {
                    let err = format!("EntryAspect::LinkAdd hold error: {}", e);
                    hold_result = Err(HolochainError::ErrorGeneric(err));
                    None
                }
            }
        }
        EntryAspect::LinkRemove((link_data, links_to_remove), _header) => Some(
            links_to_remove
                .iter()
                .fold(new_store, |mut store, link_addresses| {
                    let _ = reduce_add_remove_link_inner(
                        &mut store,
                        &link_data,
                        link_addresses,
                        LinkModification::Remove,
                    );
                    store
                }),
        ),
        EntryAspect::Update(entry, header) => {
            if let Some(crud_link) = header.link_update_delete() {
                let _ = reduce_update_entry_inner(&mut new_store, &crud_link, &entry.address());
                Some(new_store)
            } else {
                let err = "EntryAspect::Update without crud_link in header received!";
                hold_result = Err(HolochainError::ErrorGeneric(err.to_string()));
                None
            }
        }
        EntryAspect::Deletion(header) => {
            if let Some(crud_link) = header.link_update_delete() {
                let _ =
                    reduce_remove_entry_inner(&mut new_store, &crud_link, &header.entry_address());
                Some(new_store)
            } else {
                let err = "EntryAspect::Deletion without crud_link in header received!";
                hold_result = Err(HolochainError::ErrorGeneric(err.to_string()));
                None
            }
        }
        EntryAspect::Header(_) => {
            let err = "Got EntryAspect::Header which is not implemented.";
            hold_result = Err(HolochainError::ErrorGeneric(err.to_string()));
            None
        }
    };
    if let Some(ref mut store) = r {
        store.mark_aspect_as_held(&aspect);
        store.mark_hold_aspect_complete(id.clone(), hold_result);
        r
    } else {
        let mut store = (*old_store).clone();
        store.mark_hold_aspect_complete(id.clone(), hold_result);
        Some(store)
    }
}

#[allow(dead_code)]
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub(crate) fn reduce_get_links(
    _old_store: &DhtStore,
    _action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    // FIXME
    None
}

#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_queue_holding_workflow(
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let action = action_wrapper.action();
    let (pending, maybe_delay) = unwrap_to!(action => Action::QueueHoldingWorkflow);

    // TODO: TRACING: this is where we would include a Span, so that we can resume
    // the trace when the workflow gets popped (see instance.rs), but we can't do that
    // until we stop cloning the State, because Spans are not Cloneable.

    let entry_aspect = EntryAspect::from((**pending).clone());
    if old_store.get_holding_map().contains(&entry_aspect) {
        error!("Tried to add pending validation to queue which is already held!");
        None
    } else {
        if old_store.has_same_queued_holding_worfkow(pending) {
            warn!("Tried to add pending validation to queue which is already queued!");
            None
        } else {
            let mut new_store = (*old_store).clone();
            new_store
                .queued_holding_workflows
                .push_back(PendingValidationWithTimeout::new(
                    pending.clone(),
                    maybe_delay.map(ValidationTimeout::from),
                ));
            Some(new_store)
        }
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_prune(old_store: &DhtStore, _action_wrapper: &ActionWrapper) -> Option<DhtStore> {
    let pruned_queue = old_store
        .queued_holding_workflows
        .iter()
        .unique_by(|p| {
            (
                p.pending.workflow.clone(),
                p.pending.entry_with_header.header.entry_address(),
            )
        })
        .cloned()
        .collect::<VecDeque<_>>();

    if pruned_queue.len() < old_store.queued_holding_workflows.len() {
        let mut new_store = (*old_store).clone();
        new_store.queued_holding_workflows = pruned_queue;
        Some(new_store)
    } else {
        None
    }
}

#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_remove_queued_holding_workflow(
    old_store: &DhtStore,
    action_wrapper: &ActionWrapper,
) -> Option<DhtStore> {
    let action = action_wrapper.action();
    let pending = unwrap_to!(action => Action::RemoveQueuedHoldingWorkflow);
    let mut new_store = (*old_store).clone();
    if let None = new_store.remove_holding_workflow(pending) {
        error!("Got Action::PopNextHoldingWorkflow on an empty holding queue!");
    }

    Some(new_store)
}

#[cfg(test)]
pub mod tests {

    use crate::{
        action::{Action, ActionWrapper},
        content_store::{AddContent, GetContent},
        dht::{
            dht_reducers::{
                reduce, reduce_hold_aspect, reduce_queue_holding_workflow,
                reduce_remove_queued_holding_workflow,
            },
            dht_store::{create_get_links_eavi_query, DhtStore},
            pending_validations::{PendingValidation, PendingValidationStruct, ValidatingWorkflow},
        },
        instance::tests::test_context,
        network::entry_with_header::EntryWithHeader,
        state::test_store,
    };
    use bitflags::_core::time::Duration;
    use holochain_core_types::{
        agent::{test_agent_id, test_agent_id_with_name},
        chain_header::{test_chain_header, test_chain_header_with_sig},
        eav::Attribute,
        entry::{test_entry, test_sys_entry, Entry},
        link::{link_data::LinkData, Link, LinkActionKind},
        network::entry_aspect::EntryAspect,
    };
    use holochain_persistence_api::cas::content::{Address, AddressableContent};
    use snowflake::ProcessUniqueId;
    use std::{sync::Arc, time::SystemTime};

    // TODO do this for all crate tests somehow
    #[allow(dead_code)]
    fn enable_logging_for_test() {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "trace");
        }
        let _ = env_logger::builder()
            .default_format_timestamp(false)
            .default_format_module_path(false)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn reduce_hold_aspect_test() {
        let context = test_context("bob", None);
        let store = test_store(context);

        // test_entry is not sys so should do nothing
        let sys_entry = test_sys_entry();

        let new_dht_store = reduce_hold_aspect(
            &store.dht(),
            &ActionWrapper::new(Action::HoldAspect((
                EntryAspect::Content(sys_entry.clone(), test_chain_header()),
                (ProcessUniqueId::new(), ProcessUniqueId::new()),
            ))),
        )
        .expect("there should be a new store for committing a sys entry");

        assert_eq!(
            Some(sys_entry.clone()),
            store.dht().get(&sys_entry.address()).unwrap()
        );

        assert_eq!(
            Some(sys_entry.clone()),
            new_dht_store
                .get(&sys_entry.address())
                .expect("could not fetch from cas")
        );
    }

    #[test]
    fn can_add_links() {
        enable_logging_for_test();
        let context = test_context("bob", None);
        let store = test_store(context.clone());
        let entry = test_entry();

        let _ = (*store.dht()).clone().add(&entry);
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
        let action = ActionWrapper::new(Action::HoldAspect((
            EntryAspect::LinkAdd(link_data.clone(), test_chain_header()),
            (ProcessUniqueId::new(), ProcessUniqueId::new()),
        )));
        let link_entry = Entry::LinkAdd(link_data.clone());

        let new_dht_store = (*reduce(store.dht(), &action)).clone();

        let get_links_query =
            create_get_links_eavi_query(entry.address(), Some(test_link), Some(test_tag))
                .expect("supposed to create link query");
        let fetched = new_dht_store.fetch_eavi(&get_links_query);
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
        let entry = test_entry();

        let _ = (*store.dht()).clone().add(&entry);
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
        let action_link_add = ActionWrapper::new(Action::HoldAspect((
            EntryAspect::LinkAdd(link_data.clone(), test_chain_header()),
            (ProcessUniqueId::new(), ProcessUniqueId::new()),
        )));

        let new_dht_store = reduce(store.dht(), &action_link_add);

        let link_remove_data = LinkData::from_link(
            &link.clone(),
            LinkActionKind::REMOVE,
            test_chain_header(),
            test_agent_id(),
        );

        //remove added link from dht
        let action_link_remove = ActionWrapper::new(Action::HoldAspect((
            EntryAspect::LinkRemove(
                (
                    link_remove_data.clone(),
                    vec![entry_link_add.clone().address()],
                ),
                test_chain_header(),
            ),
            (ProcessUniqueId::new(), ProcessUniqueId::new()),
        )));
        let new_dht_store = reduce(new_dht_store, &action_link_remove);

        //fetch from dht and when tombstone is found return tombstone
        let get_links_query = create_get_links_eavi_query(
            entry.address(),
            Some(test_link.clone()),
            Some(test_tag.clone()),
        )
        .expect("supposed to create link query");
        let fetched = new_dht_store.fetch_eavi(&get_links_query);

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
        let action_link_add = ActionWrapper::new(Action::HoldAspect((
            EntryAspect::LinkAdd(link_data.clone(), test_chain_header()),
            (ProcessUniqueId::new(), ProcessUniqueId::new()),
        )));
        let new_dht_store = reduce(store.dht(), &action_link_add);

        //fetch from dht after link with same chain header is added
        let get_links_query = create_get_links_eavi_query(
            entry.address(),
            Some(test_link.clone()),
            Some(test_tag.clone()),
        )
        .expect("supposed to create link query");
        let fetched = new_dht_store.fetch_eavi(&get_links_query);

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
        let action_link_add = ActionWrapper::new(Action::HoldAspect((
            EntryAspect::LinkAdd(link_data.clone(), test_chain_header()),
            (ProcessUniqueId::new(), ProcessUniqueId::new()),
        )));
        let new_dht_store_2 = reduce(store.dht(), &action_link_add);

        //after new link has been added return from fetch and make sure tombstone and new link is added
        let get_links_query =
            create_get_links_eavi_query(entry.address(), Some(test_link), Some(test_tag))
                .expect("supposed to create link query");
        let fetched = new_dht_store_2.fetch_eavi(&get_links_query);

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
        let action = ActionWrapper::new(Action::HoldAspect((
            EntryAspect::LinkAdd(link_data.clone(), test_chain_header()),
            (ProcessUniqueId::new(), ProcessUniqueId::new()),
        )));

        let new_dht_store = reduce(store.dht(), &action);

        let get_links_query =
            create_get_links_eavi_query(entry.address(), Some(test_link), Some(test_tag))
                .expect("supposed to create link query");
        let fetched = new_dht_store.fetch_eavi(&get_links_query);
        assert!(fetched.is_ok());
        let hash_set = fetched.unwrap();
        assert_eq!(hash_set.len(), 0);
    }

    // TODO: Bring the old in-memory network up to speed and turn on this test again!
    #[cfg(feature = "broken-tests")]
    #[test]
    #[cfg(feature = "broken-tests")]
    pub fn reduce_hold_test() {
        let context = test_context("bill", None);
        let store = test_store(context.clone());

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::HoldAspect((
            EntryAspect::Content(entry.clone(), test_chain_header()),
            (ProcessUniqueId::new(), ProcessUniqueId::new()),
        )));

        store.reduce(action_wrapper);

        let cas = context.dht_storage.read().unwrap();

        let maybe_json = cas.fetch(&entry.address()).unwrap();
        let result_entry = match maybe_json {
            Some(content) => Entry::try_from(content).unwrap(),
            None => panic!("Could not find received entry in CAS"),
        };

        assert_eq!(&entry, &result_entry,);
    }

    fn create_pending_validation(
        entry: Entry,
        workflow: ValidatingWorkflow,
        link_update_delete: Option<Address>,
    ) -> PendingValidation {
        let entry_with_header = EntryWithHeader {
            entry: entry.clone(),
            header: test_chain_header_with_sig("sig", link_update_delete),
        };

        Arc::new(PendingValidationStruct::new(entry_with_header, workflow))
    }

    #[test]
    pub fn test_holding_queue() {
        let context = test_context("test", None);
        let store = DhtStore::new(context.dht_storage.clone(), context.eav_storage.clone());
        assert_eq!(store.queued_holding_workflows().len(), 0);

        let test_entry = test_entry();
        let hold =
            create_pending_validation(test_entry.clone(), ValidatingWorkflow::HoldEntry, None);
        let hold_header = hold.entry_with_header.header.clone();
        let action = ActionWrapper::new(Action::QueueHoldingWorkflow((
            hold.clone(),
            Some((SystemTime::now(), Duration::from_secs(10000))),
        )));
        let store = reduce_queue_holding_workflow(&store, &action).unwrap();

        assert_eq!(store.queued_holding_workflows().len(), 1);
        assert!(store.has_exact_queued_holding_workflow(&hold));

        let test_link = String::from("test_link");
        let test_tag = String::from("test-tag");
        let link = Link::new(
            &test_entry.address(),
            &test_entry.address(),
            &test_link.clone(),
            &test_tag.clone(),
        );
        let link_data = LinkData::from_link(
            &link,
            LinkActionKind::ADD,
            test_chain_header(),
            test_agent_id(),
        );

        let link_entry = Entry::LinkAdd(link_data.clone());
        let hold_link = create_pending_validation(link_entry, ValidatingWorkflow::HoldLink, None);
        let action = ActionWrapper::new(Action::QueueHoldingWorkflow((hold_link.clone(), None)));
        let store = reduce_queue_holding_workflow(&store, &action).unwrap();

        assert_eq!(store.queued_holding_workflows().len(), 2);
        assert!(store.has_exact_queued_holding_workflow(&hold_link));

        // the link won't validate while the entry is pending so we have to remove it
        let action = ActionWrapper::new(Action::RemoveQueuedHoldingWorkflow(hold.clone()));
        let store = reduce_remove_queued_holding_workflow(&store, &action).unwrap();

        let (next_pending, _) = store.next_queued_holding_workflow().unwrap();
        assert_eq!(hold_link, next_pending);

        let update = create_pending_validation(
            test_entry.clone(),
            ValidatingWorkflow::UpdateEntry,
            Some(hold_header.address()),
        );
        let action = ActionWrapper::new(Action::QueueHoldingWorkflow((update.clone(), None)));
        let store = reduce_queue_holding_workflow(&store, &action).unwrap();

        assert_eq!(store.queued_holding_workflows().len(), 2);
        assert!(!store.has_exact_queued_holding_workflow(&hold));
        assert!(store.has_exact_queued_holding_workflow(&update));
        assert!(store.has_exact_queued_holding_workflow(&hold_link));

        let action = ActionWrapper::new(Action::RemoveQueuedHoldingWorkflow(hold_link.clone()));
        let store = reduce_remove_queued_holding_workflow(&store, &action).unwrap();

        assert_eq!(store.queued_holding_workflows().len(), 1);
        assert!(!store.has_exact_queued_holding_workflow(&hold));
        assert!(!store.has_exact_queued_holding_workflow(&hold_link));
        assert!(store.has_exact_queued_holding_workflow(&update));

        let (next_pending, _) = store.next_queued_holding_workflow().unwrap();
        assert_eq!(update, next_pending);
    }
}
