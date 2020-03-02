use crate::{
    agent::state::create_entry_with_header_for_header, content_store::GetContent,

};
use holochain_logging::prelude::*;
#[autotrace]
pub mod fetch;
#[autotrace]
pub mod lists;
#[autotrace]
pub mod query;
#[autotrace]
pub mod send;
#[autotrace]
pub mod store;

use crate::{
    context::Context,
    entry::CanPublish,
    network::{
        direct_message::DirectMessage,
        entry_aspect::EntryAspect,
        entry_with_header::EntryWithHeader,
        handler::{
            fetch::*,
            lists::{handle_get_authoring_list, handle_get_gossip_list},
            query::*,
            send::*,
            store::*,
        },
    },
    workflows::get_entry_result::get_entry_with_meta_workflow,
};
use boolinator::Boolinator;
use holochain_core_types::{
    chain_header::ChainHeader, eav::Attribute, entry::Entry, error::HolochainError, time::Timeout,
};
use holochain_json_api::json::JsonString;
use holochain_net::connection::net_connection::NetHandler;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use lib3h_protocol::{
    data_types::{DirectMessageData, GenericResultData, StoreEntryAspectData},
    protocol_server::Lib3hServerProtocol,
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
        log_debug!(context, "net/handle: ignoring, same id");
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
        JsonString::from_json(std::str::from_utf8(&*data.entry_aspect.aspect.clone()).unwrap());
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
        dna_adr = data.space_address,
        provider_agent_id = data.provider_agent_id,
        entry_address = data.entry_address,
        aspect_address = data.entry_aspect.aspect_address,
        type_hint = data.entry_aspect.type_hint,
        aspect = aspect
    )
}

// See comment on fn format_store_data() - same reason for this function.
fn format_message_data(data: &DirectMessageData) -> String {
    let message_json = JsonString::from_json(std::str::from_utf8(&*data.content.clone()).unwrap());
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
        dna_adr = data.space_address,
        to = data.to_agent_id,
        from = data.from_agent_id,
        content = message,
    )
}

// TODO Implement a failure workflow?
#[autotrace]
fn handle_failure_result(
    context: &Arc<Context>,
    failure_data: GenericResultData,
) -> Result<(), HolochainError> {
    log_warn!(
        context,
        "handle_failure_result: unhandle failure={:?}",
        failure_data
    );
    Ok(())
}

/// Creates the network handler.
/// The returned closure is called by the network thread for every network event that core
/// has to handle.
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn create_handler(c: &Arc<Context>, my_dna_address: String) -> NetHandler {
    let context = c.clone();
    NetHandler::new(Box::new(move |message| {
        if let Err(err) = message {
            log_warn!(
                context,
                "net/handle: received error msg from lib3h server: {:?}",
                err
            );
            return Ok(());
        }
        let message = message.unwrap();
        let mut span = ht::SpanWrap::from(message.clone())
            .follower(&context.tracer, "received message from handler")
            .unwrap_or_else(|| {
                context
                    .tracer
                    .span("create_handler (missing history)")
                    .start()
                    .into()
            });
        span.event(format!("message.data: {:?}", message.data));
        // Set this as the root span for autotrace
        let _guard = ht::push_span(span);
        match message.data {
            Lib3hServerProtocol::FailureResult(failure_data) => {
                if !is_my_dna(&my_dna_address, &failure_data.space_address.to_string()) {
                    return Ok(());
                }

                log_warn!(context, "net/handle: FailureResult: {:?}", failure_data);
                handle_failure_result(&context, failure_data).expect("handle_failure_result")
            }
            Lib3hServerProtocol::HandleStoreEntryAspect(dht_entry_data) => {
                if !is_my_dna(&my_dna_address, &dht_entry_data.space_address.to_string()) {
                    return Ok(());
                }
                log_debug!(
                    context,
                    "net/handle: HandleStoreEntryAspect: {}",
                    format_store_data(&dht_entry_data)
                );
                handle_store(dht_entry_data, context.clone())
            }
            Lib3hServerProtocol::HandleFetchEntry(fetch_entry_data) => {
                if !is_my_dna(&my_dna_address, &fetch_entry_data.space_address.to_string()) {
                    return Ok(());
                }
                log_debug!(
                    context,
                    "net/handle: HandleFetchEntry: {:?}",
                    fetch_entry_data
                );
                handle_fetch_entry(Arc::clone(&context), fetch_entry_data)
            }
            Lib3hServerProtocol::FetchEntryResult(fetch_result_data) => {
                if !is_my_dna(
                    &my_dna_address,
                    &fetch_result_data.space_address.to_string(),
                ) {
                    return Ok(());
                }

                log_error!(
                    context,
                    "net/handle: unexpected HandleFetchEntryResult: {:?}",
                    fetch_result_data
                );
            }
            Lib3hServerProtocol::HandleQueryEntry(query_entry_data) => {
                if !is_my_dna(&my_dna_address, &query_entry_data.space_address.to_string()) {
                    return Ok(());
                }
                log_debug!(
                    context,
                    "net/handle: HandleQueryEntry: {:?}",
                    query_entry_data
                );
                handle_query_entry_data(query_entry_data, context.clone())
            }
            Lib3hServerProtocol::QueryEntryResult(query_entry_result_data) => {
                if !is_my_dna(
                    &my_dna_address,
                    &query_entry_result_data.space_address.to_string(),
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
                log_debug!(
                    context,
                    "net/handle: HandleQueryEntryResult: {:?}",
                    query_entry_result_data
                );
                handle_query_entry_result(query_entry_result_data, context.clone())
            }
            Lib3hServerProtocol::HandleSendDirectMessage(message_data) => {
                if !is_my_dna(&my_dna_address, &message_data.space_address.to_string()) {
                    ht::with_top(|span| span.event("not my dna"));
                    return Ok(());
                }
                // ignore if it's not addressed to me
                if !is_my_id(&context, &message_data.to_agent_id.to_string()) {
                    ht::with_top(|span| span.event("not my id"));
                    return Ok(());
                }
                log_debug!(
                    context,
                    "net/handle: HandleSendMessage: {}",
                    format_message_data(&message_data)
                );
                handle_send_message(message_data, context.clone())
            }
            Lib3hServerProtocol::SendDirectMessageResult(message_data) => {
                if !is_my_dna(&my_dna_address, &message_data.space_address.to_string()) {
                    return Ok(());
                }
                // ignore if it's not addressed to me
                if !is_my_id(&context, &message_data.to_agent_id.to_string()) {
                    return Ok(());
                }
                log_debug!(
                    context,
                    "net/handle: SendMessageResult: {}",
                    format_message_data(&message_data)
                );
                handle_send_message_result(message_data, context.clone())
            }
            Lib3hServerProtocol::Connected(peer_data) => {
                log_debug!(context, "net/handle: Connected: {:?}", peer_data);
                return Ok(());
            }
            Lib3hServerProtocol::HandleGetAuthoringEntryList(get_list_data) => {
                if !is_my_dna(&my_dna_address, &get_list_data.space_address.to_string()) {
                    return Ok(());
                }
                // ignore if it's not addressed to me
                if !is_my_id(&context, &get_list_data.provider_agent_id.to_string()) {
                    return Ok(());
                }

                handle_get_authoring_list(get_list_data, context.clone());
            }
            Lib3hServerProtocol::HandleGetGossipingEntryList(get_list_data) => {
                if !is_my_dna(&my_dna_address, &get_list_data.space_address.to_string()) {
                    return Ok(());
                }
                // ignore if it's not addressed to me
                if !is_my_id(&context, &get_list_data.provider_agent_id.to_string()) {
                    return Ok(());
                }

                handle_get_gossip_list(get_list_data, context.clone());
            }
            _ => {}
        }
        Ok(())
    }))
}

/// Get content aspect at this address, regardless of whether the address points to
/// an Entry or a Header
///
/// NB: this can be optimized by starting with a CAS lookup for the entry directly,
/// to avoid traversing the chain unnecessarily in the case of a miss
/// (https://github.com/holochain/holochain-rust/pull/1727#discussion_r330258624)
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn get_content_aspect(
    entry_address: &Address,
    context: Arc<Context>,
) -> Result<EntryAspect, HolochainError> {
    let state = context.state().ok_or_else(|| {
        HolochainError::InitializationFailed(String::from("In get_content_aspect: no state found"))
    })?;

    // Optimistically look for entry in chain...
    let maybe_chain_header = state
        .agent()
        .iter_chain()
        // First we look to see if the address corresponds to a header itself, if so return the header
        .find(|ref chain_header| chain_header.address() == *entry_address)
        .map(|h| (h, true))
        .or_else(|| {
            state
                .agent()
                .iter_chain()
                // Otherwise, try to find the header for the entry at this address
                .find(|ref chain_header| chain_header.entry_address() == entry_address)
                .map(|h| (h, false))
        });

    // If we have found a header for the requested entry in the chain...
    let maybe_entry_with_header = match maybe_chain_header {
        Some((header, true)) => Some(create_entry_with_header_for_header(&state, header)?),
        Some((header, false)) => {
            // ... we can just get the content from the chain CAS
            Some(EntryWithHeader {
                entry: state
                    .agent()
                    .chain_store()
                    .get(&header.entry_address())?
                    .expect("Could not find entry in chain CAS, but header is chain"),
                header,
            })
        }
        None => {
            // ... but if we didn't author that entry, let's see if we have it in the DHT cas:
            if let Some(entry) = state.dht().get(entry_address)? {
                // If we have it in the DHT cas that's good,
                // but then we have to get the header like this:
                let headers = state.get_headers(entry_address.clone()).map_err(|error| {
                    let err_message = format!(
                        "net/fetch/get_content_aspect: Error trying to get headers {:?}",
                        error
                    );
                    log_error!(context, "{}", err_message);
                    HolochainError::ErrorGeneric(err_message)
                })?;
                if !headers.is_empty() {
                    // TODO: this is just taking the first header..
                    // We should actually transform all headers into EntryAspect::Headers and just the first one
                    // into an EntryAspect content (What about ordering? Using the headers timestamp?)
                    Some(EntryWithHeader {
                        entry,
                        header: headers[0].clone(),
                    })
                } else {
                    debug!(
                        "GET CONTENT ASPECT: entry found in cas, but then couldn't find a header"
                    );
                    None
                }
            } else {
                debug!("GET CONTENT ASPECT: entry not found in cas");
                None
            }
        }
    };

    let entry_with_header = maybe_entry_with_header.ok_or(HolochainError::EntryNotFoundLocally)?;

    let _ = entry_with_header
        .entry
        .entry_type()
        .can_publish(&context)
        .ok_or(HolochainError::EntryIsPrivate)?;

    Ok(EntryAspect::Content(
        entry_with_header.entry,
        entry_with_header.header,
    ))
}

/// This function converts an entry into the right "meta" EntryAspect and the according
/// base address to which it is meta, if the entry is the source entry of a meta aspect,
/// i.e. a CRUD or link entry.
/// If the entry is not that it returns None.
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn entry_to_meta_aspect(entry: Entry, header: ChainHeader) -> Option<(Address, EntryAspect)> {
    match entry {
        Entry::App(app_type, app_value) => header.link_update_delete().map(|updated_entry| {
            (
                updated_entry,
                EntryAspect::Update(Entry::App(app_type, app_value), header),
            )
        }),
        Entry::LinkAdd(link_data) => Some((
            link_data.link.base().clone(),
            EntryAspect::LinkAdd(link_data, header),
        )),
        Entry::LinkRemove((link_data, addresses)) => Some((
            link_data.link.base().clone(),
            EntryAspect::LinkRemove((link_data, addresses), header),
        )),
        Entry::Deletion(_) => Some((
            header.link_update_delete().expect(""),
            EntryAspect::Deletion(header),
        )),
        _ => None,
    }
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn get_meta_aspects_from_chain(
    entry_address: &Address,
    context: Arc<Context>,
) -> Result<Vec<EntryAspect>, HolochainError> {
    let state = context.state().ok_or_else(|| {
        HolochainError::InitializationFailed(String::from(
            "In get_meta_aspects_from_chain: no state found",
        ))
    })?;

    Ok(state
        .agent()
        .iter_chain()
        .filter(|header| header.entry_type().can_publish(&context))
        .filter_map(
            |header| match state.agent().chain_store().get(&header.entry_address()) {
                Ok(maybe_entry) => {
                    let entry = maybe_entry
                        .expect("Could not find entry in chain CAS, but header is chain");
                    entry_to_meta_aspect(entry, header)
                }
                Err(_) => None,
            },
        )
        .filter_map(|(base_address, aspect)| {
            if base_address == *entry_address {
                Some(aspect)
            } else {
                None
            }
        })
        .collect::<Vec<EntryAspect>>())
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn get_meta_aspects_from_dht_eav(
    context: Arc<Context>,
    entry_address: &Address,
) -> Result<Vec<EntryAspect>, HolochainError> {
    let eavis = context
        .state()
        .expect("Could not get state for handle_fetch_entry")
        .dht()
        .get_all_metas(entry_address)?;

    let (aspects, errors): (Vec<_>, Vec<_>) = eavis
        .iter()
        .filter(|eavi| match eavi.attribute() {
            Attribute::LinkTag(_, _) => true,
            Attribute::RemovedLink(_, _) => true,
            Attribute::CrudLink => true,
            _ => false,
        })
        .map(|eavi| {
            let value_entry = context
                .block_on(get_entry_with_meta_workflow(
                    Arc::clone(&context),
                    &eavi.value(),
                    &Timeout::default(),
                ))?
                .ok_or_else(|| {
                    HolochainError::from("Entry linked in EAV not found! This should never happen.")
                })?;
            let header = value_entry.headers[0].to_owned();

            match eavi.attribute() {
                Attribute::LinkTag(_, _) => {
                    let link_data = unwrap_to!(value_entry.entry_with_meta.entry => Entry::LinkAdd);
                    Ok(EntryAspect::LinkAdd(link_data.clone(), header))
                }
                Attribute::RemovedLink(_, _) => {
                    let (link_data, removed_link_entries) =
                        unwrap_to!(value_entry.entry_with_meta.entry => Entry::LinkRemove);
                    Ok(EntryAspect::LinkRemove(
                        (link_data.clone(), removed_link_entries.clone()),
                        header,
                    ))
                }
                Attribute::CrudLink => Ok(EntryAspect::Update(
                    value_entry.entry_with_meta.entry,
                    header,
                )),
                _ => unreachable!(),
            }
        })
        .partition(Result::is_ok);

    if !errors.is_empty() {
        Err(errors[0].to_owned().err().unwrap())
    } else {
        Ok(aspects.into_iter().map(Result::unwrap).collect())
    }
}
