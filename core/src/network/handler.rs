use crate::{context::Context, dht::actions::hold::hold_entry, network::EntryWithHeader};
use futures::executor::block_on;
use holochain_core_types::entry::Entry;
use holochain_net_connection::{net_connection::NetHandler, protocol_wrapper::ProtocolWrapper};
use std::{convert::TryFrom, sync::Arc};

pub fn create_handler(c: &Arc<Context>) -> NetHandler {
    let context = c.clone();
    Box::new(move |message| {
        println!("ON AGENT: {:?}", context.agent);
        println!("HANDLING: {:?}", message);
        let message = message.unwrap();
        let protocol_wrapper = ProtocolWrapper::try_from(message);
        match protocol_wrapper {
            Ok(ProtocolWrapper::StoreDht(dht_data)) => {
                println!("GOT DHT STORE: {:?}", dht_data);
                let entry_with_header: EntryWithHeader =
                    serde_json::from_str(&serde_json::to_string(&dht_data.content).unwrap())
                        .unwrap();
                let maybe_address = block_on(hold_entry(
                    &entry_with_header.entry.deserialize(),
                    &context.clone(),
                ));
                println!("STORED {:?}", maybe_address);
            }
            _ => {}
        }
        Ok(())
    })
}
