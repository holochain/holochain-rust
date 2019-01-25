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
use holochain_net_connection::{json_protocol::JsonProtocol, net_connection::NetHandler};
use std::{convert::TryFrom, sync::Arc};

// FIXME: Temporary hack to ignore messages incorrectly sent to us by the networking
// module that aren't really meant for us
fn is_my_dna(context: &Arc<Context>, dna_address: &Address) -> bool {
    // TODO: we also need a better way to easily get the DNA hash!!
    let state = context
        .state()
        .ok_or("is_my_dna() could not get application state".to_string())
        .unwrap();
    let dna = state
        .nucleus()
        .dna()
        .ok_or("is_my_dna() called without DNA".to_string())
        .unwrap();
    let my_dna_address = dna.address();

    if my_dna_address != *dna_address {
        context.log("debug/net/handle: ignoring, wasn't for me");
        return false;
    }
    true
}

// FIXME: Temporary hack to ignore messages incorrectly sent to us by the networking
// module that aren't really meant for us
fn is_my_id(context: &Arc<Context>, agent_id: &str) -> bool {
    if agent_id != "" && context.agent_id.key != agent_id {
        context.log("debug/net/handle: ignoring, wasn't for me");
        return false;
    }
    true
}

// FIXME: Temporary hack to ignore messages incorrectly sent to us by the networking
// module that aren't really meant for us
fn is_for_me(context: &Arc<Context>, dna_address: &Address, agent_id: &str) -> bool {
    !is_my_id(context, agent_id) && is_my_dna(context, dna_address)
}

/// Creates the network handler.
/// The returned closure is called by the network thread for every network event that core
/// has to handle.
pub fn create_handler(c: &Arc<Context>) -> NetHandler {
    let context = c.clone();
    Box::new(move |message| {
        let message = message.unwrap();
        //context.log(format!("debug/net/handle: {:?}", message));
        let maybe_json_msg = JsonProtocol::try_from(message);
        if let Err(_) = maybe_json_msg {
            // context.log(format!("debug/net/handle: Received non-json message"));
            return Ok(());
        }
        match maybe_json_msg.unwrap() {
            JsonProtocol::HandleStoreEntry(dht_data) => {
                // NOTE data in message doesn't allow us to confirm agent!
                if !is_my_dna(&context, &dht_data.dna_address) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleStoreDhtData: {:?}",
                    dht_data
                ));
                handle_store_dht(dht_data, context.clone())
            }
            JsonProtocol::HandleStoreMeta(dht_meta_data) => {
                context.log(format!(
                    "debug/net/handle: HandleStoreDhtMeta: {:?}",
                    dht_meta_data
                ));
                if !is_my_dna(&context, &dht_meta_data.dna_address) {
                    return Ok(());
                }
                handle_store_dht_meta(dht_meta_data, context.clone())
            }
            JsonProtocol::HandleFetchEntry(fetch_dht_data) => {
                if !is_for_me(
                    &context,
                    &fetch_dht_data.dna_address,
                    &fetch_dht_data.requester_agent_id,
                ) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleFetchDhtData: {:?}",
                    fetch_dht_data
                ));
                handle_get_dht(fetch_dht_data, context.clone())
            }
            JsonProtocol::FetchEntryResult(dht_data) => {
                if !is_for_me(&context, &dht_data.dna_address, &dht_data.provider_agent_id) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: FetchDhtDataResult: {:?}",
                    dht_data
                ));
                handle_get_dht_result(dht_data, context.clone())
            }
            JsonProtocol::HandleFetchMeta(get_dht_meta_data) => {
                if is_for_me(
                    &context,
                    &get_dht_meta_data.dna_address,
                    &get_dht_meta_data.requester_agent_id,
                ) {
                    context.log(format!(
                        "debug/net/handle: HandleFetchDhtMeta: {:?}",
                        get_dht_meta_data
                    ));
                    handle_get_dht_meta(get_dht_meta_data, context.clone())
                }
            }
            JsonProtocol::FetchMetaResult(get_dht_meta_data) => {
                if is_for_me(
                    &context,
                    &get_dht_meta_data.dna_address,
                    &get_dht_meta_data.provider_agent_id,
                ) {
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
            JsonProtocol::HandleSendMessage(message_data) => {
                if !is_for_me(
                    &context,
                    &message_data.dna_address,
                    &message_data.from_agent_id,
                ) {
                    return Ok(());
                }
                handle_send(message_data, context.clone())
            }
            JsonProtocol::SendMessageResult(message_data) => {
                if !is_for_me(
                    &context,
                    &message_data.dna_address,
                    &message_data.to_agent_id,
                ) {
                    return Ok(());
                }
                handle_send_result(message_data, context.clone())
            }
            JsonProtocol::PeerConnected(peer_data) => {
                // if this is the peer connection of myself, also ignore
                if is_my_id(&context, &peer_data.agent_id) {
                    return Ok(());
                }
                // Total hack in lieu of a world-model.
                // Just republish everything when a new person comes on-line!!
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
