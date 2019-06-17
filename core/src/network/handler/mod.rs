pub mod fetch;
pub mod query;
pub mod send;
pub mod store;

use crate::{
    context::Context,
    entry::CanPublish,
    network::{
        actions::publish::publish,
        handler::{fetch::*, query::*, send::*, store::*},
    },
};
use holochain_core_types::hash::HashString;
use holochain_net::connection::{json_protocol::JsonProtocol, net_connection::NetHandler};

use std::{convert::TryFrom, sync::Arc};

// FIXME: Temporary hack to ignore messages incorrectly sent to us by the networking
// module that aren't really meant for us
fn is_my_dna(my_dna_address: &String, dna_address: &String) -> bool {
    my_dna_address == dna_address
}

// FIXME: Temporary hack to ignore messages incorrectly sent to us by the networking
// module that aren't really meant for us
fn is_my_id(context: &Arc<Context>, agent_id: &str) -> bool {
    if agent_id != "" && context.agent_id.pub_sign_key != agent_id {
        context.log("debug/net/handle: ignoring, same id");
        return false;
    }
    true
}

/// Creates the network handler.
/// The returned closure is called by the network thread for every network event that core
/// has to handle.
pub fn create_handler(c: &Arc<Context>, my_dna_address: String) -> NetHandler {
    let context = c.clone();
    NetHandler::new(Box::new(move |message| {
        let message = message.unwrap();
        // context.log(format!(
        //   "trace/net/handle:({}): {:?}",
        //   context.agent_id.nick, message
        // ));

        let maybe_json_msg = JsonProtocol::try_from(message);
        if let Err(_) = maybe_json_msg {
            return Ok(());
        }
        match maybe_json_msg.unwrap() {
            JsonProtocol::FailureResult(failure_data) => {
                if !is_my_dna(&my_dna_address, &failure_data.dna_address.to_string()) {
                    return Ok(());
                }
                context.log(format!(
                    "warning/net/handle: FailureResult: {:?}",
                    failure_data
                ));
                // TODO: Handle the reception of a FailureResult
            }
            JsonProtocol::HandleStoreEntryAspect(dht_entry_data) => {
                if !is_my_dna(&my_dna_address, &dht_entry_data.dna_address.to_string()) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleStoreEntryAspect: {:?}",
                    dht_entry_data
                ));
                handle_store(dht_entry_data, context.clone())
            }
            JsonProtocol::HandleFetchEntry(fetch_entry_data) => {
                if !is_my_dna(&my_dna_address, &fetch_entry_data.dna_address.to_string()) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleFetchEntry: {:?}",
                    fetch_entry_data
                ));
                handle_fetch_entry(fetch_entry_data, context.clone())
            }
            JsonProtocol::HandleFetchEntryResult(fetch_result_data) => {
                if !is_my_dna(&my_dna_address, &fetch_result_data.dna_address.to_string()) {
                    return Ok(());
                }

                // CLEANUP: requester_agent_id was dropped when we moved from FetchMetaResultData
                // to FetchEntryResultData, so I'm not sure if there is some other check we
                // should be doing here...
                // ignore if I'm not the requester
                //if !is_my_id(&context, &fetch_result_data.requester_agent_id.to_string()) {
                //    return Ok(());
                //}
                context.log(format!(
                    "err/net/handle: unexpected HandleFetchEntryResult: {:?}",
                    fetch_result_data
                ));
                //   handle_fetch_entry_result(fetch_result_data, context.clone())
            }
            JsonProtocol::HandleQueryEntry(query_entry_data) => {
                if !is_my_dna(&my_dna_address, &query_entry_data.dna_address.to_string()) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleQueryEntry: {:?}",
                    query_entry_data
                ));
                handle_query_entry_data(query_entry_data, context.clone())
            }
            JsonProtocol::HandleQueryEntryResult(query_entry_result_data) => {
                if !is_my_dna(
                    &my_dna_address,
                    &query_entry_result_data.dna_address.to_string(),
                ) {
                    return Ok(());
                }
                // ignore if I'm not the requester
                if !is_my_id(&context, &query_entry_result_data.requester_agent_id.to_string()) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleQueryEntryResult: {:?}",
                    query_entry_result_data
                ));
                handle_query_entry_result(query_entry_result_data, context.clone())
            }
            JsonProtocol::HandleSendMessage(message_data) => {
                if !is_my_dna(&my_dna_address, &message_data.dna_address.to_string()) {
                    return Ok(());
                }
                // ignore if it's not addressed to me
                if !is_my_id(&context, &message_data.to_agent_id.to_string()) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleSendMessage: {:?}",
                    message_data
                ));
                handle_send_message(message_data, context.clone())
            }
            JsonProtocol::SendMessageResult(message_data) => {
                if !is_my_dna(&my_dna_address, &message_data.dna_address.to_string()) {
                    return Ok(());
                }
                // ignore if it's not addressed to me
                if !is_my_id(&context, &message_data.to_agent_id.to_string()) {
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
                if is_my_id(&context, &peer_data.agent_id.to_string()) {
                    return Ok(());
                }

                context.log(format!("debug/net/handle: PeerConnected: {:?}", peer_data));
                // Total hack in lieu of a world-model.
                // Just republish everything when a new person comes on-line!!
                republish_all_public_chain_entries(&context);
            }
            _ => {}
        }
        Ok(())
    }))
}

fn republish_all_public_chain_entries(context: &Arc<Context>) {
    let chain = context.state().unwrap().agent().chain_store();
    let top_header = context.state().unwrap().agent().top_chain_header();
    chain
        .iter(&top_header)
        .filter(|ref chain_header| chain_header.entry_type().can_publish(context))
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
