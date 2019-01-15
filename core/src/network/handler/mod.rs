pub mod get;
pub mod send;
pub mod store;

use crate::{
    context::Context,
    network::{
        actions::publish::publish,
        handler::{get::*, send::*, store::*},
    },
};
use futures::executor::block_on;
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    hash::HashString,
};
use holochain_net_connection::{net_connection::NetHandler, protocol_wrapper::ProtocolWrapper};
use std::{convert::TryFrom, sync::Arc};

// FIXME: Temporary hack to ignore messages incorrectly sent to us by the networking
// module that aren't really meant for us:
fn is_me(c: &Arc<Context>, dna_address: &Address, agent_id: &str) -> bool {
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
    let my_dna_address = dna.address();

    if my_dna_address != *dna_address {
        return false;
    }
    if (my_dna_address != *dna_address) || (agent_id != "" && c.agent_id.key != agent_id) {
        c.log("debug/net/handle: ignoring, wasn't for me");
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
        //context.log(format!("debug/net/handle: {:?}", message));
        let protocol_wrapper = ProtocolWrapper::try_from(message);
        match protocol_wrapper {
            Ok(ProtocolWrapper::StoreDht(dht_data)) => {
                // NOTE data in message doesn't allow us to confirm agent!
                if !is_me(&context, &dht_data.dna_address, "") {
                    return Ok(());
                }
                context.log(format!("debug/net/handle: StoreDht: {:?}", dht_data));
                handle_store_dht(dht_data, context.clone())
            }
            Ok(ProtocolWrapper::StoreDhtMeta(dht_meta_data)) => {
                context.log(format!(
                    "debug/net/handle: StoreDhtMeta: {:?}",
                    dht_meta_data
                ));
                if !is_me(&context, &dht_meta_data.dna_address, "") {
                    context.log(format!(
                        "debug/net/handle: StoreDhtMeta: ignoring, not for me. {:?}",
                        dht_meta_data
                    ));
                    return Ok(());
                }
                handle_store_dht_meta(dht_meta_data, context.clone())
            }
            Ok(ProtocolWrapper::GetDht(get_dht_data)) => {
                // NOTE data in message doesn't allow us to confirm agent!
                if !is_me(&context, &get_dht_data.dna_address, "") {
                    return Ok(());
                }
                context.log(format!("debug/net/handle: GetDht: {:?}", get_dht_data));
                handle_get_dht(get_dht_data, context.clone())
            }
            Ok(ProtocolWrapper::GetDhtResult(dht_data)) => {
                if !is_me(&context, &dht_data.dna_address, &dht_data.agent_id) {
                    return Ok(());
                }
                context.log(format!("debug/net/handle: GetDhtResult: {:?}", dht_data));
                handle_get_dht_result(dht_data, context.clone())
            }
            Ok(ProtocolWrapper::GetDhtMeta(get_dht_meta_data)) => {
                if is_me(&context, &get_dht_meta_data.dna_address, "") {
                    context.log(format!(
                        "debug/net/handle: GetDhtMeta: {:?}",
                        get_dht_meta_data
                    ));
                    handle_get_dht_meta(get_dht_meta_data, context.clone())
                }
            }
            Ok(ProtocolWrapper::GetDhtMetaResult(get_dht_meta_data)) => {
                if is_me(&context, &get_dht_meta_data.dna_address, "") {
                    // TODO: Find a proper solution for selecting DHT meta responses.
                    // Current network implementation broadcasts messages to all nodes which means
                    // we respond to ourselves first in most cases.
                    // Eric and I thought the filter below (ignoring messages from ourselves)
                    // would fix this but that breaks several tests since in most tests
                    // we only have one instance and have to rely on the nodes local knowledge.
                    // A proper solution has to implement some aspects of what we call the
                    // "world model". A node needs to know what context it's in: if we are the only
                    // node we know about (like in these tests) we can not ignore our local knowledge
                    // but in other cases we should rather rely on the network's response.
                    // In the end this needs a full CRDT implemention.
                    //if is_me(
                    //    &context,
                    //    &get_dht_meta_data.dna_address,
                    //    &get_dht_meta_data.from_agent_id,
                    //) {
                    //    context.log("debug/net/handle: Got DHT meta result from myself. Ignoring.");
                    //    return Ok(());
                    //} else {
                    context.log(format!(
                        "debug/net/handle: GetDhtMetaResult: {:?}",
                        get_dht_meta_data
                    ));
                    handle_get_dht_meta_result(get_dht_meta_data, context.clone())
                    //}
                }
            }
            Ok(ProtocolWrapper::HandleSend(message_data)) => {
                if !is_me(
                    &context,
                    &message_data.dna_address,
                    &message_data.to_agent_id,
                ) {
                    return Ok(());
                }
                handle_send(message_data, context.clone())
            }
            Ok(ProtocolWrapper::SendResult(message_data)) => {
                if !is_me(
                    &context,
                    &message_data.dna_address,
                    &message_data.to_agent_id,
                ) {
                    return Ok(());
                }
                handle_send_result(message_data, context.clone())
            }
            Ok(ProtocolWrapper::PeerConnected(peer_data)) => {
                // if is not my DNA ignore
                if !is_me(&context, &peer_data.dna_address, "") {
                    return Ok(());
                }
                // if this is the peer connection of myself, also ignore
                if is_me(&context, &peer_data.dna_address, &peer_data.agent_id) {
                    return Ok(());
                }
                // Total hack in lieu of a world-model.  Just republish everything
                // when a new person comes on-line!!
                republish_all_public_chain_entries(&context);
            }
            _ => {}
        }
        Ok(())
    })
}

fn republish_all_public_chain_entries(context: &Arc<Context>) {
    let chain = context.state().unwrap().agent().chain();
    let top_header = context.state().unwrap().agent().top_chain_header();
    chain
        .iter(&top_header)
        .filter(|ref chain_header| chain_header.entry_type().can_publish())
        .for_each(|chain_header| {
            let hash = HashString::from(chain_header.entry_address().to_string());
            match block_on(publish(hash.clone(), context)) {
                Err(e) => context.log(format!(
                    "err/net/handle: unable to publish {:?}, got error: {:?}",
                    hash, e
                )),
                _ => {}
            }
        });
}
