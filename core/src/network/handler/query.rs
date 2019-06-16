use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    entry::CanPublish,
    instance::dispatch_action,
    network::query::NetworkQuery,
    nucleus,
};
use holochain_core_types::{cas::content::Address, eav::Attribute, entry::EntryWithMetaAndHeader, json::JsonString};
use holochain_net::connection::json_protocol::{
    QueryEntryData, FetchEntryData, FetchEntryResultData, QueryEntryResultData,
};
use std::{collections::BTreeSet, convert::TryInto, sync::Arc};

/// The network has sent us a query for entry data, so we need to examine
/// the query and create appropriate actions for the different variants
pub fn handle_query_entry_data(fetch_meta_data: QueryEntryData, context: Arc<Context>) {
    let query_json = JsonString::from(fetch_meta_data.query);
    match query_json.try_into() {
        Ok(NetworkQuery::GetLinks(link_type, tag)) => {
            let links = context
                .state()
                .unwrap()
                .dht()
                .get_links(
                    Address::from(fetch_meta_data.entry_address.clone()),
                    link_type.clone(),
                    tag.clone(),
                )
                .unwrap_or(BTreeSet::new())
                .into_iter()
                .map(|eav| eav.value())
                .collect::<Vec<_>>();
            let action_wrapper = ActionWrapper::new(Action::RespondGetLinks((fetch_meta_data, links)));
            dispatch_action(context.action_channel(), action_wrapper.clone());
        },
        _ => panic!(format!("handle query entry data variant not implemented: {:?}",query_json)),
    }
}

/// The network comes back with a result to our previous query with a result, so we
/// examine the query result for its type and dispatch different actions according to variant
pub fn handle_query_entry_result(dht_meta_data: QueryEntryResultData, context: Arc<Context>) {
    let query_result_json = JsonString::from(dht_meta_data.query_result);
    match query_result_json.try_into() {
        Ok(Attribute::LinkTag(link_type, tag)) => {
            let action_wrapper = ActionWrapper::new(Action::HandleGetLinksResult((
                dht_meta_data,
                link_type,
                tag,
            )));
            dispatch_action(context.action_channel(), action_wrapper.clone());
        },
        _ => panic!(format!("handle query entry result variant not implemented: {:?}",query_result_json)),
    }
}
