use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::actions::{add_link::add_link, hold::hold_entry},
    network::util::EntryWithHeader,
    nucleus,
    instance::dispatch_action,
};
use futures::executor::block_on;
use holochain_core_types::{
    cas::content::Address,
    entry::Entry,
};
use holochain_net_connection::{net_connection::NetHandler, protocol_wrapper::ProtocolWrapper};
use std::{convert::TryFrom, sync::Arc};

pub fn create_handler(c: &Arc<Context>) -> NetHandler {
    let context = c.clone();
    Box::new(move |message| {
        let message = message.unwrap();
        let protocol_wrapper = ProtocolWrapper::try_from(message);
        match protocol_wrapper {
            Ok(ProtocolWrapper::StoreDht(dht_data)) => {
                let entry_with_header: EntryWithHeader =
                    serde_json::from_str(&serde_json::to_string(&dht_data.content).unwrap())
                        .unwrap();
                let _ = block_on(hold_entry(&entry_with_header.entry, &context.clone()));
            }
            Ok(ProtocolWrapper::StoreDhtMeta(dht_meta_data)) => {
                let entry_with_header: EntryWithHeader =
                    serde_json::from_str(&serde_json::to_string(&dht_meta_data.content).unwrap())
                        .unwrap();
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
            },
            Ok(ProtocolWrapper::GetDht(get_dht_data)) => {
                let _ = block_on(nucleus::actions::get_entry::get_entry(&context, Address::from(get_dht_data.address.clone())))
                    .map(|maybe_entry| {
                        let action_wrapper = ActionWrapper::new(Action::RespondGet((get_dht_data, maybe_entry)));
                        dispatch_action(&context.action_channel, action_wrapper.clone());
                    });
            },
            _ => {}
        }
        Ok(())
    })
}
