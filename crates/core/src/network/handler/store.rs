use crate::{
    context::Context,
    dht::{
        actions::queue_holding_workflow::dispatch_queue_holding_workflow,
        pending_validations::PendingValidationStruct,
    },NEW_RELIC_LICENSE_KEY
};
use holochain_core_types::network::entry_aspect::EntryAspect;
use holochain_json_api::json::JsonString;
use lib3h_protocol::data_types::StoreEntryAspectData;
use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};

/// The network requests us to store (i.e. hold) the given entry aspect data.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn handle_store(dht_data: StoreEntryAspectData, context: Arc<Context>) {
    let aspect_json =
        JsonString::from_json(std::str::from_utf8(&*dht_data.entry_aspect.aspect).unwrap());
    let maybe_aspect: Result<EntryAspect, _> = aspect_json.clone().try_into();
    if let Ok(aspect) = maybe_aspect {
        if context
            .state()
            .unwrap()
            .dht()
            .get_holding_map()
            .contains(&aspect)
        {
            log_error!(
                context,
                "handle_store: Aspect already being held: {:?}",
                aspect
            );
            return;
        }
        match PendingValidationStruct::try_from(aspect) {
            Err(e) => log_error!(
                context,
                "net/handle: handle_store: received bad aspect: {:?}",
                e,
            ),
            Ok(pending) => {
                log_debug!(
                    context,
                    "net/handle: handle_store: Adding {} to holding queue...",
                    pending.workflow,
                );
                dispatch_queue_holding_workflow(Arc::new(pending), None, context);
            }
        }
    } else {
        log_error!(
            context,
            "net/handle_store: Unable to parse entry aspect: {}",
            aspect_json
        )
    }
}

/*
/// The network requests us to store meta information (links/CRUD/etc) for an
/// entry that we hold.
pub fn handle_store_meta(dht_meta_data: DhtMetaData, context: Arc<Context>) {
    let attr = dht_meta_data.clone().attribute;
    // @TODO: If network crates will switch to using the `Attribute` enum,
    // we can match on the enum directly
    if attr == Attribute::Link.to_string() {
        log_debug!(context, "net/handle: HandleStoreMeta: got LINK. processing...");
        // TODO: do a loop on content once links properly implemented
        assert_eq!(dht_meta_data.content_list.len(), 1);
        let entry_with_header: EntryWithHeader = serde_json::from_str(
            &serde_json::to_string(&dht_meta_data.content_list[0])
                .expect("dht_meta_data should be EntryWithHeader"),
        )
        .expect("dht_meta_data should be EntryWithHeader");
        thread::spawn(move || {
            match context.block_on(hold_link_workflow(&entry_with_header, &context.clone())) {
                Err(error) => log_error!(context, "net/dht: {}", error),
                _ => (),
            }
        });
    } else if attr == Attribute::LinkRemove.to_string() {
        log_debug!(context, "net/handle: HandleStoreMeta: got LINK REMOVAL. processing...");
        // TODO: do a loop on content once links properly implemented
        assert_eq!(dht_meta_data.content_list.len(), 1);
        let entry_with_header: EntryWithHeader = serde_json::from_str(
            //should be careful doing slice access, it might panic
            &serde_json::to_string(&dht_meta_data.content_list[0])
                .expect("dht_meta_data should be EntryWithHader"),
        )
        .expect("dht_meta_data should be EntryWithHader");
        thread::spawn(move || {
            if let Err(error) =
                context.block_on(remove_link_workflow(&entry_with_header, &context.clone()))
            {
                log_error!(context, "net/dht: {}", error)
            }
        });
    } else if CrudStatus::from_str(&attr)
        .expect("Could not convert deleted attribute to CrudStatus")
        == CrudStatus::Deleted
    {
        log_debug!(context, "net/handle: HandleStoreMeta: got CRUD STATUS. processing...");

        let entry_with_header: EntryWithHeader = serde_json::from_str(
            //should be careful doing slice access, it might panic
            &serde_json::to_string(&dht_meta_data.content_list[0])
                .expect("dht_meta_data should be EntryWithHader"),
        )
        .expect("dht_meta_data should be EntryWithHader");
        thread::spawn(move || {
            if let Err(error) =
                context.block_on(hold_remove_workflow(entry_with_header, context.clone()))
            {
                log_error!(context, "net/dht: {}", error)
            }
        });
    } else if CrudStatus::from_str(&attr)
        .expect("Could not convert modified attribute to CrudStatus")
        == CrudStatus::Modified
    {
        log_debug!(context, "net/handle: HandleStoreMeta: got CRUD LINK. processing...");
        let entry_with_header: EntryWithHeader = serde_json::from_str(
            //should be careful doing slice access, it might panic
            &serde_json::to_string(&dht_meta_data.content_list[0])
                .expect("dht_meta_data should be EntryWithHader"),
        )
        .expect("dht_meta_data should be EntryWithHader");
        thread::spawn(move || {
            if let Err(error) =
                context.block_on(hold_update_workflow(entry_with_header, context.clone()))
            {
                log_error!(context, "net/dht: {}", error)
            }
        });
    }
}
*/
