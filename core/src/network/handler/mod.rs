pub mod get;
pub mod send;
pub mod store;

use crate::{
    context::Context,
    network::handler::{get::*, send::*, store::*},
};
use holochain_net_connection::{net_connection::NetHandler, protocol_wrapper::ProtocolWrapper};
use std::{convert::TryFrom, sync::Arc};

// FIXME: Temporary hack to ignore messages incorrectly sent to us by the networking
// module that aren't really meant for us:
fn is_me(c: &Arc<Context>, dna_hash: &str, agent_id: &str) -> bool {
    // TODO: we also need a better way to easily get the DNA hash!!
    let state = c
        .state()
        .ok_or("is_me could not get application state".to_string())
        .unwrap();
    let dna = state
        .nucleus()
        .dna()
        .ok_or("is_me called without DNA".to_string())
        .unwrap();
    let my_dna_hash = base64::encode(&dna.multihash().unwrap());

    if my_dna_hash != dna_hash {
        return false;
    }
    if (my_dna_hash != dna_hash) || (agent_id != "" && c.agent_id.key != agent_id) {
        c.log("HANDLE: ignoring, wasn't for me");
        false
    } else {
        true
    }
}

/// Creates the network handler.
/// The returned closure is called by the network thread for every network event that core
/// has to handle.
pub fn create_handler(c: &Arc<Context>) -> NetHandler {
    let context = c.clone();
    Box::new(move |message| {
        let message = message.unwrap();
        context.log(format!("HANDLE: {:?}", message));
        let protocol_wrapper = ProtocolWrapper::try_from(message);
        match protocol_wrapper {
            Ok(ProtocolWrapper::StoreDht(dht_data)) => {
                // NOTE data in message doesn't allow us to confirm agent!
                if !is_me(&context, &dht_data.dna_hash, "") {
                    return Ok(());
                }
                handle_store_dht(dht_data, context.clone())
            }
            Ok(ProtocolWrapper::StoreDhtMeta(dht_meta_data)) => {
                if !is_me(&context, &dht_meta_data.dna_hash, "") {
                    return Ok(());
                }
                handle_store_dht_meta(dht_meta_data, context.clone())
            }
            Ok(ProtocolWrapper::GetDht(get_dht_data)) => {
                // NOTE data in message doesn't allow us to confirm agent!
                if !is_me(&context, &get_dht_data.dna_hash, "") {
                    return Ok(());
                }
                handle_get_dht(get_dht_data, context.clone())
            }
            Ok(ProtocolWrapper::GetDhtResult(dht_data)) => {
                if !is_me(&context, &dht_data.dna_hash, &dht_data.agent_id) {
                    return Ok(());
                }
                handle_get_dht_result(dht_data, context.clone())
            }
            Ok(ProtocolWrapper::HandleSend(message_data)) => {
                if !is_me(&context, &message_data.dna_hash, &message_data.to_agent_id) {
                    return Ok(());
                }
                handle_send(message_data, context.clone())
            }
            Ok(ProtocolWrapper::SendResult(message_data)) => {
                if !is_me(&context, &message_data.dna_hash, &message_data.to_agent_id) {
                    return Ok(());
                }
                handle_send_result(message_data, context.clone())
            }
            _ => {}
        }
        Ok(())
    })
}
