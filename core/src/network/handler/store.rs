use crate::{
    context::Context,
    network::entry_with_header::EntryWithHeader,
    workflows::{hold_entry::hold_entry_workflow, hold_link::hold_link_workflow},
};
use futures::executor::block_on;
use holochain_core_types::{
    cas::content::Address,
    crud_status::{CrudStatus, LINK_NAME, STATUS_NAME},
};
use holochain_net_connection::protocol_wrapper::{DhtData, DhtMetaData};
use std::{sync::Arc, thread};

/// The network requests us to store (i.e. hold) the given entry.
pub fn handle_store_dht(dht_data: DhtData, context: Arc<Context>) {
    let entry_with_header: EntryWithHeader =
        serde_json::from_str(&serde_json::to_string(&dht_data.content).unwrap()).unwrap();
    thread::spawn(move || {
        match block_on(hold_entry_workflow(&entry_with_header, &context.clone())) {
            Err(error) => context.log(format!("err/net/dht: {}", error)),
            _ => (),
        }
    });
}

/// The network requests us to store meta information (links/CRUD/etc) for an
/// entry that we hold.
pub fn handle_store_dht_meta(dht_meta_data: DhtMetaData, context: Arc<Context>) {
    match dht_meta_data.attribute.as_ref() {
        "link" => {
            context.log("HANDLE: StoreDhtMeta: got LINK. processing...");
            let entry_with_header: EntryWithHeader = serde_json::from_str(
                &serde_json::to_string(&dht_meta_data.content)
                    .expect("dht_meta_data should be EntryWithHader"),
            )
            .expect("dht_meta_data should be EntryWithHader");
            thread::spawn(move || {
                match block_on(hold_link_workflow(&entry_with_header, &context.clone())) {
                    Err(error) => context.log(format!("err/net/dht: {}", error)),
                    _ => (),
                }
            });
        }
        STATUS_NAME => {
            context.log("HANDLE: StoreDhtMeta: got CRUD status. processing...");
            let _crud_status: CrudStatus = serde_json::from_str(
                &serde_json::to_string(&dht_meta_data.content)
                    .expect("dht_meta_data should be crud_status"),
            )
            .expect("dht_meta_data should be crud_status");
            // FIXME: block_on hold crud_status metadata in DHT?
        }
        LINK_NAME => {
            context.log("HANDLE: StoreDhtMeta: got CRUD LINK. processing...");
            let _crud_link: Address = serde_json::from_str(
                &serde_json::to_string(&dht_meta_data.content)
                    .expect("dht_meta_data should be crud_link"),
            )
            .expect("dht_meta_data should be crud_link");
            // FIXME: block_on hold crud_link metadata in DHT?
        }
        _ => {}
    }
}
