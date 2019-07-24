use crate::{
    action::{Action, ActionWrapper, GetEntryKey, GetLinksKey},
    context::Context,
    entry::CanPublish,
    instance::dispatch_action,
    network::query::{
        GetLinksNetworkQuery, GetLinksNetworkResult, NetworkQuery, NetworkQueryResult,
    },
    nucleus,
};
use holochain_core_types::{crud_status::CrudStatus, entry::EntryWithMetaAndHeader};
use holochain_json_api::json::JsonString;
use holochain_persistence_api::cas::content::Address;
use lib3h_protocol::data_types::{QueryEntryData, QueryEntryResultData};
use std::{collections::BTreeSet, convert::TryInto, sync::Arc};

fn get_links(
    context: &Arc<Context>,
    base: Address,
    link_type: String,
    tag: String,
    crud_status: Option<CrudStatus>,
) -> Vec<(Address, CrudStatus)> {
    context
        .state()
        .unwrap()
        .dht()
        .get_links(base, link_type, tag, crud_status)
        .unwrap_or(BTreeSet::new())
        .into_iter()
        .map(|eav_crud| (eav_crud.0.value(), eav_crud.1))
        .collect::<Vec<_>>()
}

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
                                    entry_with_meta: entry_with_meta.clone(),
                                    headers,
                                })
                            } else {
                                None
                            }
                        })
                        .unwrap_or(None)
                })
                .map_err(|error| {
                    context.log(format!("err/net: Error trying to get headers {:?}", error));
                    None::<EntryWithMetaAndHeader>
                })
        })
        .map_err(|error| {
            context.log(format!("err/net: Error trying to find entry {:?}", error));
            None::<EntryWithMetaAndHeader>
        })
        .unwrap_or(Ok(None))
        .unwrap_or(None)
}

/// The network has sent us a query for entry data, so we need to examine
/// the query and create appropriate actions for the different variants
pub fn handle_query_entry_data(query_data: QueryEntryData, context: Arc<Context>) {
    let query_json = JsonString::from_json(&String::from_utf8(query_data.query.clone()).unwrap());
    let action_wrapper = match query_json.clone().try_into() {
        Ok(NetworkQuery::GetLinks(link_type, tag, options, query)) => {
            let links = get_links(
                &context,
                query_data.entry_address.clone(),
                link_type.clone(),
                tag.clone(),
                options,
            );
            let links_result = match query {
                GetLinksNetworkQuery::Links => GetLinksNetworkResult::Links(links),
                GetLinksNetworkQuery::Count => GetLinksNetworkResult::Count(links.len()),
            };

            ActionWrapper::new(Action::RespondGetLinks((
                query_data,
                links_result,
                link_type.clone(),
                tag.clone(),
            )))
        }
        Ok(NetworkQuery::GetEntry) => {
            let maybe_entry = get_entry(&context, query_data.entry_address.clone());
            ActionWrapper::new(Action::RespondGet((query_data, maybe_entry)))
        }
        err => {
            context.log(format!(
                "err/net: Error ({:?}) deserializing Query {:?}",
                err, query_json
            ));
            return;
        }
    };
    dispatch_action(context.action_channel(), action_wrapper);
}

/// The network comes back with a result to our previous query with a result, so we
/// examine the query result for its type and dispatch different actions according to variant
pub fn handle_query_entry_result(query_result_data: QueryEntryResultData, context: Arc<Context>) {
    let query_result_json =
        JsonString::from_json(&String::from_utf8(query_result_data.query_result).unwrap());
    let action_wrapper = match query_result_json.clone().try_into() {
        Ok(NetworkQueryResult::Entry(maybe_entry)) => {
            ActionWrapper::new(Action::HandleGetResult((
                maybe_entry,
                GetEntryKey {
                    address: query_result_data.entry_address.clone(),
                    id: query_result_data.request_id.clone(),
                },
            )))
        }
        Ok(NetworkQueryResult::Links(links_result, link_type, tag)) => {
            ActionWrapper::new(Action::HandleGetLinksResult((
                links_result,
                GetLinksKey {
                    base_address: query_result_data.entry_address.clone(),
                    link_type: link_type.clone(),
                    tag: tag.clone(),
                    id: query_result_data.request_id.clone(),
                },
            )))
        }
        err => {
            context.log(format!(
                "err/net: Error ({:?}) deserializing QueryResult {:?}",
                err, query_result_json
            ));
            return;
        }
    };
    dispatch_action(context.action_channel(), action_wrapper.clone());
}
