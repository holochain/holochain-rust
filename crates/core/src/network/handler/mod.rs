use crate::{agent::state::create_entry_with_header_for_header, content_store::GetContent};
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
        handler::{
            fetch::*,
            lists::{handle_get_authoring_list, handle_get_gossip_list},
            query::*,
            send::*,
            store::*,
        },
    },
    workflows::get_entry_result::get_entry_with_meta_workflow_local,
};
use holochain_core_types::{
    chain_header::ChainHeader, eav::Attribute, entry::Entry, error::HolochainError,
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
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
                handle_fetch_entry(fetch_entry_data, context.clone())
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

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn get_content_aspects_from_chain(
    entry_address: &Address,
    context: Arc<Context>,
) -> Result<Vec<EntryAspect>, HolochainError> {
    let state = context.state().ok_or_else(|| {
        HolochainError::InitializationFailed(String::from(
            "In get_content_aspects_from_chain: no state found",
        ))
    })?;

    let aspects = state
        .agent()
        .iter_chain()
        .filter_map(|ref chain_header| {
            if !chain_header.entry_type().can_publish(&context) {
                return None;
            };
            if chain_header.address() == *entry_address {
                if let Ok(ewh) = create_entry_with_header_for_header(&state, chain_header.clone()) {
                    Some(EntryAspect::Content(ewh.entry, ewh.header))
                } else {
                    None
                }
            } else if chain_header.entry_address() == entry_address {
                if let Ok(maybe_entry) = state.agent().chain_store().get(&entry_address) {
                    if let Some(entry) = maybe_entry {
                        Some(EntryAspect::Content(entry, chain_header.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    Ok(aspects)
}

/// Get the content aspects from the cas for an address, regardless of whether the address points to
/// an Entry or a Header. There can be more than one if the entry was committed twice either
/// by the same agent or by multiple agents.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn get_content_aspects(
    entry_address: &Address,
    context: Arc<Context>,
) -> Result<Vec<EntryAspect>, HolochainError> {
    let state = context.state().ok_or_else(|| {
        HolochainError::InitializationFailed(String::from("In get_content_aspects: no state found"))
    })?;

    let mut aspects: Vec<EntryAspect> = Vec::new();

    if let Some(entry) = state.dht().get(entry_address)? {
        // If we have it in the DHT cas that's good,
        // but then we have to get the header like this:
        let headers = state.get_headers(entry_address.clone()).map_err(|error| {
            let err_message = format!(
                "net/fetch/get_content_aspects: Error trying to get headers {:?}",
                error
            );
            log_error!(context, "{}", err_message);
            HolochainError::ErrorGeneric(err_message)
        })?;
        if !headers.is_empty() {
            for h in headers {
                aspects.push(EntryAspect::Content(entry.clone(), h.clone()));
            }
        } else {
            error!("GET CONTENT ASPECTS: entry found in cas, but then couldn't find a header");
        }
    } else {
        error!("GET CONTENT ASPECTS: entry not found in cas");
    }
    Ok(aspects)
}

/// This function converts an entry into the right "meta" EntryAspect and the according
/// base address to which it is meta, if the entry is the source entry of a meta aspect,
/// i.e. a CRUD or link entry.
/// If the entry is not that it returns None.
///
/// NB: this is the inverse function of EntryAspect::entry_address(), so it is very important
/// that they agree!
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn entry_to_meta_aspect(
    entry: Entry,
    header: ChainHeader,
) -> Result<Option<(Address, EntryAspect)>, HolochainError> {
    let maybe_aspect = match entry {
        Entry::App(app_type, app_value) => header
            .link_update_delete()
            .map(|_| EntryAspect::Update(Entry::App(app_type, app_value), header)),
        Entry::LinkAdd(link_data) => Some(EntryAspect::LinkAdd(link_data, header)),
        Entry::LinkRemove((link_data, addresses)) => {
            Some(EntryAspect::LinkRemove((link_data, addresses), header))
        }
        Entry::Deletion(_) => Some(EntryAspect::Deletion(header)),
        _ => None,
    };
    if let Some(aspect) = maybe_aspect {
        Ok(Some((aspect.entry_address()?, aspect)))
    } else {
        Ok(None)
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
                        .expect("Couldn't derive meta aspect from entry")
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

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn get_meta_aspects_from_dht_eav(
    entry_address: &Address,
    context: Arc<Context>,
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
            let value_entry = get_entry_with_meta_workflow_local(&context, &eavi.value())?
                .ok_or_else(|| {
                    HolochainError::from("Entry linked in EAV not found! This should never happen.")
                })?;
            let header = value_entry.headers[0].to_owned();

            match eavi.attribute() {
                Attribute::LinkTag(_, _) => match value_entry.entry_with_meta.entry {
                    Entry::LinkAdd(link_data) | Entry::LinkRemove((link_data, _)) => {
                        Ok(EntryAspect::LinkAdd(link_data, header))
                    }
                    _ => Err(HolochainError::from(format!(
                        "Invalid Entry Value for LinkTag: {:?}",
                        value_entry
                    ))),
                },
                Attribute::RemovedLink(_link_type, _link_tag) => {
                    match value_entry.entry_with_meta.entry {
                        Entry::LinkRemove((link_data, removed_link_entries)) => Ok(
                            EntryAspect::LinkRemove((link_data, removed_link_entries), header),
                        ),
                        Entry::LinkAdd(link_data) => {
                            // here we are manually building the entry aspect assuming
                            // just one link being removed which is actually correct for holochain
                            // but currently incorrect in this implementation and needs to be fixed
                            // in hdk v3.
                            Ok(EntryAspect::LinkRemove(
                                (link_data, vec![eavi.value()]),
                                header,
                            ))
                        }
                        _ => Err(HolochainError::from(format!(
                            "Invalid Entry Value for RemovedLink: {:?}",
                            value_entry
                        ))),
                    }
                }
                Attribute::CrudLink => Ok(EntryAspect::Update(
                    value_entry.entry_with_meta.entry,
                    header,
                )),
                _ => Err(HolochainError::from(format!(
                    "Invalid Attribute in eavi: {:?}",
                    eavi.attribute()
                ))),
            }
        })
        .partition(Result::is_ok);
    if !errors.is_empty() {
        Err(errors[0].to_owned().err().unwrap())
    } else {
        Ok(aspects.into_iter().map(Result::unwrap).collect())
    }
}
