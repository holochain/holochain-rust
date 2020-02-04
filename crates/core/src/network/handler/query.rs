use crate::{
    action::{Action, ActionWrapper, GetEntryKey, GetLinksKey, QueryKey},
    context::Context,
    entry::CanPublish,
    instance::dispatch_action,
    network::query::{
        GetLinksNetworkQuery, GetLinksNetworkResult, NetworkQuery, NetworkQueryResult,
    },
    nucleus, NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::{
    crud_status::CrudStatus,
    eav::Attribute,
    entry::EntryWithMetaAndHeader,
    error::HolochainError,
    network::query::{GetLinkFromRemoteData, Pagination},
};
use holochain_json_api::json::JsonString;
use holochain_persistence_api::cas::content::Address;

use lib3h_protocol::data_types::{QueryEntryData, QueryEntryResultData};
use std::{convert::TryInto, sync::Arc};

pub type LinkTag = String;
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]

fn get_links(
    context: &Arc<Context>,
    base: Address,
    link_type: String,
    tag: String,
    crud_status: Option<CrudStatus>,
    pagination: Option<Pagination>,
) -> Result<Vec<GetLinkFromRemoteData>, HolochainError> {
    //get links
    let dht_store = context.state().unwrap().dht();
    Ok(dht_store
        .get_links(base, link_type, tag, crud_status, pagination)
        .unwrap_or_default()
        .into_iter()
        .map(|(eavi, crud_status)| {
            let tag = match eavi.attribute() {
                Attribute::LinkTag(_, tag) => Ok(tag),
                Attribute::RemovedLink(_, tag) => Ok(tag),
                _ => Err(HolochainError::ErrorGeneric(
                    "Could not get tag".to_string(),
                )),
            }
            .expect("INVALID ATTRIBUTE ON EAV GET, SOMETHING VERY WRONG IN EAV QUERY");
            GetLinkFromRemoteData {
                tag,
                crud_status,
                link_add_address: eavi.value(),
            }
        })
        .collect())
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn get_entry(context: &Arc<Context>, address: Address) -> Option<EntryWithMetaAndHeader> {
    nucleus::actions::get_entry::get_entry_with_meta(&context, address.clone())
        .map(|entry_with_meta_opt| {
            let state = context
                .state()
                .expect("Could not get state for handle_fetch_entry");
            state
                .get_headers(address)
                .map(|headers| {
                    entry_with_meta_opt
                        .map(|entry_with_meta| {
                            if entry_with_meta.entry.entry_type().can_publish(&context) {
                                Some(EntryWithMetaAndHeader {
                                    entry_with_meta: entry_with_meta,
                                    headers,
                                })
                            } else {
                                None
                            }
                        })
                        .unwrap_or(None)
                })
                .map_err(|error| {
                    log_error!(context, "net: Error trying to get headers {:?}", error);
                    None::<EntryWithMetaAndHeader>
                })
        })
        .map_err(|error| {
            log_error!(context, "net: Error trying to find entry {:?}", error);
            None::<EntryWithMetaAndHeader>
        })
        .unwrap_or(Ok(None))
        .unwrap_or(None)
}

/// The network has sent us a query for entry data, so we need to examine
/// the query and create appropriate actions for the different variants
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn handle_query_entry_data(query_data: QueryEntryData, context: Arc<Context>) {
    let query_json =
        JsonString::from_json(&std::str::from_utf8(&*query_data.query.clone()).unwrap());
    let action_wrapper = match query_json.clone().try_into() {
        Ok(NetworkQuery::GetLinks(link_type, tag, options, query)) => {
            match get_links(
                &context,
                query_data.entry_address.clone().into(),
                link_type.clone(),
                tag.clone(),
                options,
                match query.clone() {
                    GetLinksNetworkQuery::Links(get_headers) => get_headers.pagination,
                    _ => None,
                },
            ) {
                Ok(links) => {
                    let links_result = match query {
                        GetLinksNetworkQuery::Links(_) => GetLinksNetworkResult::Links(links),
                        GetLinksNetworkQuery::Count => GetLinksNetworkResult::Count(links.len()),
                    };
                    let respond_links = NetworkQueryResult::Links(links_result, link_type, tag);
                    ActionWrapper::new(Action::RespondQuery((query_data, respond_links)))
                }
                Err(err) => {
                    log_error!(
                        context,
                        "net: Error ({:?}) getting links from dht node",
                        err,
                    );
                    return;
                }
            }
        }
        Ok(NetworkQuery::GetEntry) => {
            let maybe_entry = get_entry(&context, query_data.entry_address.clone().into());
            let respond_get = NetworkQueryResult::Entry(maybe_entry);
            ActionWrapper::new(Action::RespondQuery((query_data, respond_get)))
        }
        err => {
            log_error!(
                context,
                "net: Error ({:?}) deserializing Query {:?}",
                err,
                query_json
            );
            return;
        }
    };
    dispatch_action(context.action_channel(), action_wrapper);
}

/// The network comes back with a result to our previous query with a result, so we
/// examine the query result for its type and dispatch different actions according to variant
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn handle_query_entry_result(query_result_data: QueryEntryResultData, context: Arc<Context>) {
    let query_result_json = JsonString::from_json(
        std::str::from_utf8(&*query_result_data.clone().query_result).unwrap(),
    );
    log_trace!(
        context,
        "handle_query_entry_result: {:?}",
        query_result_data
    );
    let action_wrapper = match query_result_json.clone().try_into() {
        Ok(NetworkQueryResult::Entry(maybe_entry)) => {
            let payload = NetworkQueryResult::Entry(maybe_entry);
            ActionWrapper::new(Action::HandleQuery((
                payload,
                QueryKey::Entry(GetEntryKey {
                    address: query_result_data.entry_address.clone().into(),
                    id: query_result_data.request_id.clone(),
                }),
            )))
        }
        Ok(NetworkQueryResult::Links(links_result, link_type, tag)) => {
            let payload = NetworkQueryResult::Links(links_result, link_type.clone(), tag.clone());
            ActionWrapper::new(Action::HandleQuery((
                payload,
                QueryKey::Links(GetLinksKey {
                    base_address: query_result_data.entry_address.clone().into(),
                    link_type: link_type,
                    tag: tag,
                    id: query_result_data.request_id,
                }),
            )))
        }
        err => {
            log_error!(
                context,
                "net: Error ({:?}) deserializing QueryResult {:?}",
                err,
                query_result_json
            );
            return;
        }
    };
    dispatch_action(context.action_channel(), action_wrapper);
}
