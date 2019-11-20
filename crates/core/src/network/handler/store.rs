use crate::{
    context::Context,
    network::{chain_pair::ChainPair, entry_aspect::EntryAspect},
    workflows::{
        hold_entry::hold_entry_workflow, hold_entry_remove::hold_remove_workflow,
        hold_entry_update::hold_update_workflow, hold_link::hold_link_workflow,
        remove_link::remove_link_workflow,
    },
};
use holochain_core_types::{entry::{deletion_entry::DeletionEntry, Entry}, error::HolochainError};
use holochain_json_api::json::JsonString;
use holochain_persistence_api::cas::content::AddressableContent;
use lib3h_protocol::data_types::StoreEntryAspectData;
use snowflake::ProcessUniqueId;
use std::{convert::TryInto, sync::Arc, thread};

/// The network requests us to store (i.e. hold) the given entry aspect data.
pub fn handle_store(dht_data: StoreEntryAspectData, context: Arc<Context>) {
    let aspect_json =
        JsonString::from_json(std::str::from_utf8(&*dht_data.entry_aspect.aspect).unwrap());
    if let Ok(aspect) = aspect_json.clone().try_into() {
        match aspect {
            EntryAspect::Content(entry, header) => {
                log_debug!(
                    context,
                    "net/handle: handle_store: Got EntryAspect::Content. processing..."
                );

                match ChainPair::new(header, entry) {
                    Ok(chain_pair) => {
                        thread::Builder::new()
                        .name(format!(
                            "store_entry_content/{}",
                            ProcessUniqueId::new().to_string()
                        ))
                        .spawn(move || {
                            if let Err(err) =
                                context.block_on(hold_entry_workflow(&chain_pair, context.clone()))
                            {
                                log_error!(context, "net/dht: {}", err);
                            }
                        })
                        .expect("Could not spawn thread for storing EntryAspect::Content");
                    },
                    Err(err) => {
                        log_error!(context, "net/handle: handle_store: Got EntryAspect::Content with non-matching entry {:#?} and header {:#?}", err, entry, header)
                    }
                }
            }
            EntryAspect::Header(header) => {
                panic!(format!("unimplemented store aspect Header: {:?}", header));
            }
            EntryAspect::LinkAdd(link_data, header) => {
                log_debug!(
                    context,
                    "net/handle: handle_store: Got EntryAspect::LinkAdd. processing..."
                );
                let entry = Entry::LinkAdd(link_data);
                if entry.address() != *header.entry_address() {
                    log_error!(context, "net/handle: handle_store: Got EntryAspect::LinkAdd with non-matching LinkData and ChainHeader! Hash of content in header does not match content! Ignoring.");
                    return;
                }
                match ChainPair::new(header, entry) {
                    Ok(chain_pair) => {
                        thread::Builder::new()
                        .name(format!(
                            "store_link_entry/{}",
                            ProcessUniqueId::new().to_string()
                        ))
                        .spawn(move || {
                            if let Err(error) = context
                                .block_on(hold_link_workflow(&chain_pair, context.clone()))
                            {
                                log_error!(context, "net/dht: {}", error);
                            }
                        })
                        .expect("Could not spawn thread for storing EntryAspect::LinkAdd");
                    },
                    Err(error) => {
                        log_error!(context, "net/handle: handle_store: Could not create a ChainPair for EntryAspect::LinkAdd in order to store it. Error:\n{}\nEntry:\n{:#?}\nHeader:{:#?}", error, entry, header)
                    }
                }
                
            }
            EntryAspect::LinkRemove((link_data, links_to_remove), header) => {
                log_debug!(
                    context,
                    "net/handle: handle_store: Got EntryAspect::LinkRemove. processing...",
                );
                let entry = Entry::LinkRemove((link_data, links_to_remove));
                match ChainPair::new(header, entry) {
                    Ok(chain_pair) => {
                        thread::Builder::new()
                        .name(format!(
                            "store_link_remove/{}",
                            ProcessUniqueId::new().to_string()
                        ))
                        .spawn(move || {
                            if let Err(error) = context
                                .block_on(remove_link_workflow(&chain_pair, context.clone()))
                            {
                                log_error!(context, "net/dht: {}", error)
                            }
                        })
                        .expect("Could not spawn thread for storing EntryAspect::LinkRemove");
                    },
                    Err(error) => {
                        log_error!(context, "net/handle: handle_store:
                        Couldn't not create a ChainPair in order to store the
                        entry and header for EntryAspect::LinkRemove. Error:\n
                        {}\nEntry:\n{:#?}\nHeader:\n{:#?}", 
                        error, entry, header)
                    }
                }
            }
            EntryAspect::Update(entry, header) => {
                log_debug!(
                    context,
                    "net/handle: handle_store: Got EntryAspect::Update. processing..."
                );
                match ChainPair::new(header, entry) {
                    Ok(chain_pair) => {
                        thread::Builder::new()
                        .name(format!(
                            "store_update/{}",
                            ProcessUniqueId::new().to_string()
                        ))
                        .spawn(move || {
                            if let Err(error) = context
                                .block_on(hold_update_workflow(&chain_pair, context.clone()))
                            {
                                log_error!(context, "net/dht: {}", error)
                            }
                        })
                        .expect("Could not spawn thread for storing EntryAspect::Update");
                    },
                    Err(error) => {
                        log_error!(context, "net/handle: handle_store: Could not create ChainPair in order to store EntryAspect::Update, header: {:#?}, entry: {:#?}. Error: {}. Error: {}", 
                        header, entry, error)
                    }
                } else if Err(error) {
                    log_error!(context, "net/handle: failed processing EntryAspect::Update while trying to create a chain pair, with error: {}. The entry is {}, and header is {}", error, entry, header)
                };   
            }
            EntryAspect::Deletion(header) => {
                log_debug!(
                    context,
                    "net/handle: handle_store: Got EntryAspect::Deletion. processing...",
                );
                // reconstruct the deletion entry from the header.
                let deleted_entry_address = match header.link_update_delete() {
                    None => {
                        log_error!(context, "net/handle: handle_store: Got EntryAspect::Deletion with header that has no deletion link! Ignoring.");
                        return;
                    }
                    Some(address) => address,
                };

                let entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address));
                match ChainPair::new(header, entry) {
                    Ok(chain_pair) => {
                        thread::Builder::new()
                        .name(format!(
                            "store_deletion/{}",
                            ProcessUniqueId::new().to_string()
                        ))
                        .spawn(move || {
                            if let Err(error) = context
                                .block_on(hold_remove_workflow(&chain_pair, context.clone()))
                            {
                                log_error!(context, "net/handle_store: {}", error)
                            }
                        })
                        .expect("Could not spawn thread for storing EntryAspect::Deletion");
                    },
                    Err(err) => {
                        log_error!(context, "net/handle: handle_store: Could
                        not create ChainPair in order to store
                        EntryAspect::Deletion. Entry:\n{:#?}\n. The entry in
                        the attempted chain pair is a deletion entry, and the
                        header is:\n{:#?}\nError: {}", entry, header, err)
                    },
                }
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
// If reintroducing this, update entry_with_header to chain_pair
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
