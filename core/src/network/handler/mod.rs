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
use holochain_net::connection::{json_protocol::JsonProtocol, net_connection::NetHandler};
use holochain_persistence_api::hash::HashString;

use crate::network::{direct_message::DirectMessage, entry_aspect::EntryAspect};
use holochain_json_api::json::JsonString;
use holochain_net::connection::json_protocol::{MessageData, StoreEntryAspectData};
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

// Since StoreEntryAspectData lives in the net crate and EntryAspect is specific
// to core we can't implement fmt::Debug so that it spans over both, StoreEntryAspectData
// and the type that is represented as opaque byte vector.
// For debug logs we do want to see the whole store request including the EntryAspect.
// This function enables pretty debug logs by deserializing the EntryAspect explicitly
// and combining it with the top-level fields in a formatted and indented output.
fn format_store_data(data: &StoreEntryAspectData) -> String {
    let aspect_json =
        JsonString::from_json(&String::from_utf8(data.entry_aspect.aspect.clone()).unwrap());
    let aspect = EntryAspect::try_from(aspect_json).unwrap();
    format!(
        r#"
StoreEntryAspectData {{
    request_id: "{req_id}",
    dna_address: "{dna_adr}",
    provider_agent_id: "{provider_agent_id}",
    entry_address: "{entry_address}",
    entry_aspect: {{
        aspect_address: "{aspect_address}",
        type_hint: "{type_hint}",
        aspect: "{aspect:?}"
    }}
}}"#,
        req_id = data.request_id,
        dna_adr = data.dna_address,
        provider_agent_id = data.provider_agent_id,
        entry_address = data.entry_address,
        aspect_address = data.entry_aspect.aspect_address,
        type_hint = data.entry_aspect.type_hint,
        aspect = aspect
    )
}

// See comment on fn format_store_data() - same reason for this function.
fn format_message_data(data: &MessageData) -> String {
    let message_json = JsonString::from_json(&String::from_utf8(data.content.clone()).unwrap());
    let message = DirectMessage::try_from(message_json).unwrap();
    format!(
        r#"
MessageData {{
    request_id: "{req_id}",
    dna_address: "{dna_adr}",
    to_agent_id: "{to}",
    from_agent_id: "{from}",
    content: {content:?},
}}"#,
        req_id = data.request_id,
        dna_adr = data.dna_address,
        to = data.to_agent_id,
        from = data.from_agent_id,
        content = message,
    )
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
                    "debug/net/handle: HandleStoreEntryAspect: {}",
                    format_store_data(&dht_entry_data)
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

                context.log(format!(
                    "err/net/handle: unexpected HandleFetchEntryResult: {:?}",
                    fetch_result_data
                ));
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
            JsonProtocol::QueryEntryResult(query_entry_result_data) => {
                if !is_my_dna(
                    &my_dna_address,
                    &query_entry_result_data.dna_address.to_string(),
                ) {
                    return Ok(());
                }
                // ignore if I'm not the requester
                if !is_my_id(
                    &context,
                    &query_entry_result_data.requester_agent_id.to_string(),
                ) {
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
                    "debug/net/handle: HandleSendMessage: {}",
                    format_message_data(&message_data)
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
                    "debug/net/handle: SendMessageResult: {}",
                    format_message_data(&message_data)
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

pub fn republish_all_public_chain_entries(context: &Arc<Context>) {
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
