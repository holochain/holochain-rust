use crate::{
    context::Context,
    network::entry_with_header::EntryWithHeader,
    workflows::{hold_entry::hold_entry_workflow, hold_link::hold_link_workflow},
};
use holochain_core_types::crud_status::{LINK_NAME, STATUS_NAME};
use holochain_net_connection::json_protocol::{DhtMetaData, EntryData};
use std::{sync::Arc, thread};

/// The network requests us to store (i.e. hold) the given entry.
pub fn handle_store_entry(dht_data: EntryData, context: Arc<Context>) {
    let entry_with_header: EntryWithHeader =
        serde_json::from_str(&serde_json::to_string(&dht_data.entry_content).unwrap()).unwrap();
    thread::spawn(move || {
        match context.block_on(hold_entry_workflow(entry_with_header, context.clone())) {
            Err(error) => context.log(format!("err/net/dht: {}", error)),
            _ => (),
        }
    });
}

/// The network requests us to store meta information (links/CRUD/etc) for an
/// entry that we hold.
pub fn handle_store_meta(dht_meta_data: DhtMetaData, context: Arc<Context>) {
    match dht_meta_data.attribute.as_ref() {
        "link" => {
            context.log("debug/net/handle: HandleStoreMeta: got LINK. processing...");
            let entry_with_header: EntryWithHeader = serde_json::from_str(
                &serde_json::to_string(&dht_meta_data.content_list)
                    .expect("dht_meta_data should be EntryWithHader"),
            )
            .expect("dht_meta_data should be EntryWithHader");
            thread::spawn(move || {
                match context.block_on(hold_link_workflow(&entry_with_header, &context.clone())) {
                    Err(error) => context.log(format!("err/net/dht: {}", error)),
                    _ => (),
                }
            });
        }
        STATUS_NAME => {
            context.log("debug/net/handle: HandleStoreMeta: got CRUD status. processing...");
            // FIXME: block_on hold crud_status metadata in DHT?
        }
        LINK_NAME => {
            context.log("debug/net/handle: HandleStoreMeta: got CRUD LINK. processing...");
            // FIXME: block_on hold crud_link metadata in DHT?
        }
        _ => {}
    }
}
