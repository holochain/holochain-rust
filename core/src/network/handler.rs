use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::actions::{add_link::add_link, hold::hold_entry},
    instance::dispatch_action,
    network::entry_with_header::EntryWithHeader,
    nucleus,
};
use futures::executor::block_on;
use holochain_core_types::{
    cas::content::Address,
    entry::Entry,
    crud_status::{CrudStatus, LINK_NAME, STATUS_NAME},
};
use holochain_net_connection::{net_connection::NetHandler, protocol_wrapper::ProtocolWrapper};
use std::{convert::TryFrom, sync::Arc};

pub fn create_handler(c: &Arc<Context>) -> NetHandler {
    let context = c.clone();
    Box::new(move |message| {
        let message = message.unwrap();
        let protocol_wrapper = ProtocolWrapper::try_from(message);
        //println!("create_handler() protocol_wrapper = {:?}", protocol_wrapper);
        match protocol_wrapper {
            Ok(ProtocolWrapper::StoreDht(dht_data)) => {
                let entry_with_header: EntryWithHeader =
                    serde_json::from_str(&serde_json::to_string(&dht_data.content).unwrap())
                        .unwrap();
                //println!("create_handler() hold_entry...");
                let _ = block_on(hold_entry(&entry_with_header.entry_body, &context.clone()));
            }
            Ok(ProtocolWrapper::StoreDhtMeta(dht_meta_data)) => {
                //println!("create_handler() StoreDhtMeta request received: {:?}", dht_meta_data);
                match dht_meta_data.attribute.as_ref() {
                    "link" => {
                        //println!("create_handler() StoreDhtMeta request received: Link");
                        let entry_with_header: EntryWithHeader =
                            serde_json::from_str(
                                &serde_json::to_string(&dht_meta_data.content)
                                    .expect("dht_meta_data should be EntryWithHader")
                            )
                                .expect("dht_meta_data should be EntryWithHader");
                        let link_add = match entry_with_header.entry_body {
                            Entry::LinkAdd(link_add) => link_add,
                            _ => unreachable!(),
                        };
                        let link = link_add.link().clone();
                        let _ = block_on(add_link(&link, &context.clone()));
                    }
                    STATUS_NAME => {
                        //println!("create_handler() StoreDhtMeta {}", STATUS_NAME);
                        let _crud_status: CrudStatus =
                            serde_json::from_str(
                                &serde_json::to_string(&dht_meta_data.content)
                                    .expect("dht_meta_data should be crud_status")
                            )
                                .expect("dht_meta_data should be crud_status");
                        // FIXME: block_on hold crud_status metadata in DHT?
                    }
                    LINK_NAME => {
                        //println!("create_handler() StoreDhtMeta {}", LINK_NAME);
                        let _crud_link: Address =
                            serde_json::from_str(
                                &serde_json::to_string(&dht_meta_data.content)
                                    .expect("dht_meta_data should be crud_link")
                            )
                                .expect("dht_meta_data should be crud_link");
                        // FIXME: block_on hold crud_link metadata in DHT?
                    }
                    _ => {}
                }
            }
            Ok(ProtocolWrapper::GetDht(get_dht_data)) => {
                let _ = nucleus::actions::get_entry::get_entry_with_meta(
                    &context,
                    Address::from(get_dht_data.address.clone()),
                )
                .map(|maybe_entry_with_meta| {
                    let action_wrapper = ActionWrapper::new(Action::RespondGet((
                        get_dht_data,
                        maybe_entry_with_meta,
                    )));
                    // println!("create_handler() dispatch_action = {:?}", action_wrapper);
                    dispatch_action(&context.action_channel, action_wrapper.clone());
                });
            }
            Ok(ProtocolWrapper::GetDhtResult(dht_data)) => {
                let action_wrapper = ActionWrapper::new(Action::HandleGetResult(dht_data));
                dispatch_action(&context.action_channel, action_wrapper.clone());
            }
            _ => {}
        }
        Ok(())
    })
}
