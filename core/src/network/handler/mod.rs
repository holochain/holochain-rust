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
        context.log("debug/net/handle: ignoring, not my dna");
        return false;
    }
    true
}

// FIXME: Temporary hack to ignore messages incorrectly sent to us by the networking
// module that aren't really meant for us
fn is_my_id(context: &Arc<Context>, agent_id: &str) -> bool {
    if agent_id != "" && context.agent_id.key != agent_id {
        context.log("debug/net/handle: ignoring, same id");
        return false;
    }
    true
}

/// Creates the network handler.
/// The returned closure is called by the network thread for every network event that core
/// has to handle.
pub fn create_handler(c: &Arc<Context>) -> NetHandler {
    let context = c.clone();
    Box::new(move |message| {
        let message = message.unwrap();
         context.log(format!(
             "trace/net/handle:({}): {:?}",
             context.agent_id.nick, message
         ));
        let maybe_json_msg = JsonProtocol::try_from(message);
        if let Err(_) = maybe_json_msg {
            return Ok(());
        }
        match maybe_json_msg.unwrap() {
            JsonProtocol::FailureResult(failure_data) => {
                if !is_my_dna(&context, &failure_data.dna_address) {
                    return Ok(());
                }
                context.log(format!(
                    "warning/net/handle: FailureResult: {:?}",
                    failure_data
                ));
                println!("RECEIVED FailureResult: {:?}", failure_data);
            }
            JsonProtocol::HandleStoreEntry(dht_entry_data) => {
                if !is_my_dna(&context, &dht_entry_data.dna_address) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleStoreEntry: {:?}",
                    dht_entry_data
                ));
                handle_store_entry(dht_entry_data, context.clone())
            }
            JsonProtocol::HandleStoreMeta(dht_meta_data) => {
                if !is_my_dna(&context, &dht_meta_data.dna_address) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleStoreMeta: {:?}",
                    dht_meta_data
                ));
                handle_store_meta(dht_meta_data, context.clone())
            }
            JsonProtocol::HandleFetchEntry(fetch_entry_data) => {
                if !is_my_dna(&context, &fetch_entry_data.dna_address) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleFetchEntry: {:?}",
                    fetch_entry_data
                ));
                handle_fetch_entry(fetch_entry_data, context.clone())
            }
            JsonProtocol::FetchEntryResult(fetch_result_data) => {
                if !is_my_dna(&context, &fetch_result_data.dna_address) {
                    return Ok(());
                }
                // ignore if I'm not the requester
                if !is_my_id(&context, &fetch_result_data.requester_agent_id) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: FetchEntryResult: {:?}",
                    fetch_result_data
                ));
                handle_fetch_entry_result(fetch_result_data, context.clone())
            }
            JsonProtocol::HandleFetchMeta(fetch_meta_data) => {
                if !is_my_dna(&context, &fetch_meta_data.dna_address) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleFetchMeta: {:?}",
                    fetch_meta_data
                ));
                handle_fetch_meta(fetch_meta_data, context.clone())
            }
            JsonProtocol::FetchMetaResult(fetch_meta_result_data) => {
                if !is_my_dna(&context, &fetch_meta_result_data.dna_address) {
                    return Ok(());
                }
                // ignore if I'm not the requester
                if !is_my_id(&context, &fetch_meta_result_data.requester_agent_id) {
                    return Ok(());
                }
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
                    "debug/net/handle: FetchMetaResult: {:?}",
                    fetch_meta_result_data
                ));
                handle_fetch_meta_result(fetch_meta_result_data, context.clone())
                //}
            }
            JsonProtocol::HandleSendMessage(message_data) => {
                if !is_my_dna(&context, &message_data.dna_address) {
                    return Ok(());
                }
                // ignore if it's not addressed to me
                if !is_my_id(&context, &message_data.to_agent_id) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleSendMessage: {:?}",
                    message_data
                ));
                handle_send_message(message_data, context.clone())
            }
            JsonProtocol::SendMessageResult(message_data) => {
                if !is_my_dna(&context, &message_data.dna_address) {
                    return Ok(());
                }
                // ignore if it's not addressed to me
                if !is_my_id(&context, &message_data.to_agent_id) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: SendMessageResult: {:?}",
                    message_data
                ));
                handle_send_message_result(message_data, context.clone())
            }
            JsonProtocol::PeerConnected(peer_data) => {
                // ignore peer connection of myself
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
    let chain = context.state().unwrap().agent().chain_store();
    let top_header = context.state().unwrap().agent().top_chain_header();
    chain
        .iter(&top_header)
        .filter(|ref chain_header| chain_header.entry_type().can_publish())
        .for_each(|chain_header| {
            let hash = HashString::from(chain_header.entry_address().to_string());
            match context.block_on(publish(hash.clone(), context)) {
                Err(e) => context.log(format!(
                    "err/net/handle: unable to publish {:?}, got error: {:?}",
                    hash, e
                )),
                _ => {}
            }
        });
}
