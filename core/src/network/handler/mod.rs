pub mod get;
pub mod send;
pub mod store;
use crate::{
    context::Context,
    entry::CanPublish,
    network::{
        actions::publish::publish,
        handler::{get::*, send::*, store::*},
    },
};
use holochain_core_types::{
    cas::content::Address, eav::EntityAttributeValueIndex, error::HolochainError, hash::HashString,
};
use holochain_net::connection::{
    json_protocol::{EntryListData, GetListData, JsonProtocol, MetaListData, MetaTuple},
    net_connection::{NetHandler, NetSend},
};

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
    Box::new(move |message| {
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
            JsonProtocol::HandleStoreEntry(dht_entry_data) => {
                if !is_my_dna(&my_dna_address, &dht_entry_data.dna_address.to_string()) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleStoreEntry: {:?}",
                    dht_entry_data
                ));
                handle_store_entry(dht_entry_data, context.clone())
            }
            JsonProtocol::HandleStoreMeta(dht_meta_data) => {
                if !is_my_dna(&my_dna_address, &dht_meta_data.dna_address.to_string()) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleStoreMeta: {:?}",
                    dht_meta_data
                ));
                handle_store_meta(dht_meta_data, context.clone())
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
            JsonProtocol::FetchEntryResult(fetch_result_data) => {
                if !is_my_dna(&my_dna_address, &fetch_result_data.dna_address.to_string()) {
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
                if !is_my_dna(&my_dna_address, &fetch_meta_data.dna_address.to_string()) {
                    return Ok(());
                }
                context.log(format!(
                    "debug/net/handle: HandleFetchMeta: {:?}",
                    fetch_meta_data
                ));
                handle_fetch_meta(fetch_meta_data, context.clone())
            }
            JsonProtocol::FetchMetaResult(fetch_meta_result_data) => {
                if !is_my_dna(
                    &my_dna_address,
                    &fetch_meta_result_data.dna_address.to_string(),
                ) {
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
                if !is_my_dna(&my_dna_address, &message_data.dna_address.to_string()) {
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
                if !is_my_dna(&my_dna_address, &message_data.dna_address.to_string()) {
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

                context.log(format!("debug/net/handle: PeerConnected: {:?}", peer_data));
                // Total hack in lieu of a world-model.
                // Just republish everything when a new person comes on-line!!
                republish_all_public_chain_entries(&context);
            }
            JsonProtocol::HandleGetHoldingEntryList(get_list_data) => {
                handle_get_holding_entry_list(&context, &get_list_data)
                    .expect("handle_get_holding_entry_list: failed")
             }
            JsonProtocol::HandleGetHoldingMetaList(get_list_data) => {
                handle_get_holding_meta_list(&context, &get_list_data)
                    .expect("handle_get_holding_meta_list: failed")
            }
            JsonProtocol::HandleGetPublishingEntryList(get_list_data) => {
                handle_get_publishing_entry_list(&context, &get_list_data)
                    .expect("handle_get_publish_entries: failed");
            }
            JsonProtocol::HandleGetPublishingMetaList(get_list_data) => {
                handle_get_publishing_meta_list(&context, &get_list_data)
                    .expect("handle_get_publishing_meta_list: failed");
            }
            // these protocol events should be handled on the lib3h side.
            JsonProtocol::HandleGetPublishingEntryListResult(_) |
               JsonProtocol::HandleGetHoldingEntryListResult(_) |
                JsonProtocol::HandleGetPublishingMetaListResult(_) |
                JsonProtocol::HandleGetHoldingMetaListResult(_) => (),
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

fn handle_get_publishing_meta_list(
    context: &Arc<Context>,
    get_list_data: &GetListData,
) -> Result<(), HolochainError> {
    context
        .state()
        .expect("State missing from context.")
        .dht()
        .get_meta()
        .and_then(|meta_map| {
            let meta_list: Vec<MetaTuple> = meta_map
                .into_iter()
                .map(|eavi: EntityAttributeValueIndex| {
                    let content_hash: Address = eavi.value();
                    let meta_tuple: MetaTuple = (
                        eavi.entity(),
                        eavi.attribute().to_string(),
                        serde_json::value::Value::String(content_hash.into()),
                    );
                    meta_tuple
                })
                .collect();

            let meta_list_data =
                MetaListData {
                    dna_address: get_list_data.dna_address.clone(),
                    request_id: get_list_data.request_id.clone(),
                    meta_list
                };

            send_result(
                &context,
                JsonProtocol::HandleGetPublishingMetaListResult(meta_list_data)
            )
        })
}

fn handle_get_publishing_entry_list(
    context: &Arc<Context>,
    get_list_data: &GetListData,
) -> Result<(), HolochainError> {
    let chain = context.state().unwrap().agent().chain_store();
    let top_header = context.state().unwrap().agent().top_chain_header();
    let entry_address_list = chain
        .iter(&top_header)
        .filter(|ref chain_header| chain_header.entry_type().can_publish(context))
        .map(|chain_header| {
            let address: &Address = chain_header.entry_address();
            address.clone()
        })
        .collect();
    let entry_list_data = EntryListData {
        dna_address: get_list_data.dna_address.clone(),
        request_id: get_list_data.request_id.clone(),
        entry_address_list: entry_address_list,
    };

    send_result(
        &context,
        JsonProtocol::HandleGetPublishingEntryListResult(entry_list_data),
    )
}
fn handle_get_holding_entry_list(
    _context: &Arc<Context>,
    _get_list_data: &GetListData,
) -> Result<(), HolochainError> {
    Ok(())
}

fn handle_get_holding_meta_list(
    _context: &Arc<Context>,
    _get_list_data: &GetListData,
) -> Result<(), HolochainError> {
    Ok(())
}

fn send_result(context: &Arc<Context>, json_protocol: JsonProtocol) -> Result<(), HolochainError> {
    context.log(format!(
        "debug/net/handle: sending result over network: {:?}",
        json_protocol
    ));
    let network = context
        .state()
        .expect("state should be present")
        .network()
        .network
        .clone();
    // Send the list back to the calling peer
    network
        .expect("network should be present")
        .lock()
        .expect("get network mutex")
        .send(json_protocol.into())
        .map_err(|err: failure::Error| HolochainError::new(err.to_string().as_str()))
}


