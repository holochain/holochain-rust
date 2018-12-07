use crate::{
    context::Context,
    dht::actions::{add_link::add_link, hold::hold_entry},
    network::util::EntryWithHeader,
};
use futures::executor::block_on;
use holochain_core_types::entry::Entry;
use holochain_net_connection::protocol_wrapper::{DhtData, DhtMetaData};
use std::sync::Arc;

/// The network requests us to store (i.e. hold) the given entry.
pub fn handle_store_dht(dht_data: DhtData, context: Arc<Context>) {
    let entry_with_header: EntryWithHeader =
        serde_json::from_str(&serde_json::to_string(&dht_data.content).unwrap()).unwrap();
    let _ = block_on(hold_entry(&entry_with_header.entry, &context.clone()));
}

/// The network requests us to store meta information (links/CRUD/etc) for an
/// entry that we hold.
pub fn handle_store_dht_meta(dht_meta_data: DhtMetaData, context: Arc<Context>) {
    let entry_with_header: EntryWithHeader =
        serde_json::from_str(&serde_json::to_string(&dht_meta_data.content).unwrap()).unwrap();
    match dht_meta_data.attribute.as_ref() {
        "link" => {
            let link_add = match entry_with_header.entry {
                Entry::LinkAdd(link_add) => link_add,
                _ => unreachable!(),
            };
            let link = link_add.link().clone();
            let _ = block_on(add_link(&link, &context.clone()));
        }
        _ => {}
    }
}
