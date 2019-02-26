use crate::{
    context::Context,
    network::entry_with_header::EntryWithHeader,
    workflows::{
        hold_entry::hold_entry_workflow,
        hold_entry_remove::hold_remove_workflow,
        hold_entry_update::hold_update_workflow,
        hold_link::hold_link_workflow,
        remove_link::remove_link_workflow,
    },
};
use holochain_core_types::{cas::content::Address, crud_status::CrudStatus, eav::Attribute,entry::entry_type::EntryType};
use holochain_net_connection::json_protocol::{DhtMetaData, EntryData};
use std::{sync::Arc, thread};

/// The network requests us to store (i.e. hold) the given entry.
pub fn handle_store_entry(dht_data: EntryData, context: Arc<Context>) {
    let entry_with_header: EntryWithHeader =
        serde_json::from_str(&serde_json::to_string(&dht_data.entry_content).unwrap()).unwrap();
    let entry = entry_with_header.entry.clone();
    match entry.entry_type()
    {
        EntryType::App(_) =>
        {
            if entry_with_header.header.link_update_delete().is_none()
            {
                    thread::spawn(move || {
                    match context.block_on(hold_entry_workflow(entry_with_header, context.clone())) {
                        Err(error) => context.log(format!("err/net/dht: {}", error)),
                        _ => (),
                    }
                });
            }
            else
            {
                    thread::spawn(move || {
                    match context.block_on(hold_update_workflow(entry_with_header, context.clone())) {
                        Err(error) => context.log(format!("err/net/dht: {}", error)),
                        _ => (),
                    }
                });
            }

            
        },
        EntryType::Deletion =>
        {
            thread::spawn(move || {
                match context.block_on(hold_remove_workflow(entry_with_header, context.clone())) {
                    Err(error) => context.log(format!("err/net/dht: {}", error)),
                    _ => (),
                }
            });
        },
        _ => ()
        

    }
    
}

/// The network requests us to store meta information (links/CRUD/etc) for an
/// entry that we hold.
pub fn handle_store_meta(dht_meta_data: DhtMetaData, context: Arc<Context>) {
    let attr = dht_meta_data.clone().attribute;
    // @TODO: If network crates will switch to using the `Attribute` enum,
    // we can match on the enum directly
    if attr == Attribute::Link.to_string() {
        context.log("debug/net/handle: HandleStoreMeta: got LINK. processing...");
        // TODO: do a loop on content once links properly implemented
        assert_eq!(dht_meta_data.content_list.len(), 1);
        let entry_with_header: EntryWithHeader = serde_json::from_str(
            &serde_json::to_string(&dht_meta_data.content_list[0])
                .expect("dht_meta_data should be EntryWithHeader"),
        )
        .expect("dht_meta_data should be EntryWithHeader");
        thread::spawn(move || {
            match context.block_on(hold_link_workflow(&entry_with_header, &context.clone())) {
                Err(error) => context.log(format!("err/net/dht: {}", error)),
                _ => (),
            }
        });
    } else if attr == Attribute::LinkRemove.to_string() {
        context.log("debug/net/handle: HandleStoreMeta: got LINK REMOVAL. processing...");
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
                context.log(format!("err/net/dht: {}", error))
            }
        });
    } else if attr == Attribute::CrudStatus.to_string() {
        context.log("debug/net/handle: HandleStoreMeta: got CRUD STATUS. processing...");
    // FIXME: block_on hold crud_status metadata in DHT?
       
    } else if attr == Attribute::CrudLink.to_string() {
        context.log("debug/net/handle: HandleStoreMeta: got CRUD LINK. processing...");
        // FIXME: block_on hold crud_link metadata in DHT?

    }
}
