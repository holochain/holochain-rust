use crate::{
    context::Context,
    dht::actions::{add_link::add_link, hold::hold_entry},
    network::util::EntryWithHeader,
};
use futures::executor::block_on;
use holochain_core_types::{entry::ToEntry, link::link_add::LinkAddEntry};
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
                let _ = block_on(hold_entry(
                    &entry_with_header.entry.deserialize(),
                    &context.clone(),
                ));
            }
            Ok(ProtocolWrapper::StoreDhtMeta(dht_meta_data)) => {
                let entry_with_header: EntryWithHeader =
                    serde_json::from_str(&serde_json::to_string(&dht_meta_data.content).unwrap())
                        .unwrap();
                match dht_meta_data.attribute.as_ref() {
                    "link" => {
                        let link_add_entry =
                            LinkAddEntry::from_entry(&entry_with_header.entry.deserialize());
                        let link = link_add_entry.link().clone();
                        let _ = block_on(add_link(&link, &context.clone()));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    })
}
