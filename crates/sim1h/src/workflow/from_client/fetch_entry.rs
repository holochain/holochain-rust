use crate::{
    dht::bbdht::{dynamodb::client::Client, error::BbDhtResult},
    trace::LogContext,
    workflow::from_client::query_entry::query_entry_aspects,
};
use holochain_core_types::network::query::NetworkQuery;
use holochain_json_api::json::JsonString;
use lib3h_protocol::{
    data_types::{EntryData, FetchEntryData, FetchEntryResultData, QueryEntryData},
    protocol::ClientToLib3hResponse,
};

/// MVP (needs tests, wrapping query atm)
/// query entry but hardcoded to entry query right?
pub fn fetch_entry(
    log_context: &LogContext,
    client: &Client,
    fetch_entry_data: &FetchEntryData,
) -> BbDhtResult<ClientToLib3hResponse> {
    let query_entry_data = QueryEntryData {
        request_id: fetch_entry_data.request_id.clone(),
        // seems weird but the two structs don't line up 1:1
        requester_agent_id: fetch_entry_data.provider_agent_id.clone(),
        space_address: fetch_entry_data.space_address.clone(),
        entry_address: fetch_entry_data.entry_address.clone(),
        query: JsonString::from(NetworkQuery::GetEntry).to_bytes().into(),
    };
    let query_aspect_list = query_entry_aspects(log_context, client, &query_entry_data)?;
    let fetch_entry_result_data = FetchEntryResultData {
        // i think this works??
        entry: EntryData {
            aspect_list: query_aspect_list,
            entry_address: query_entry_data.entry_address,
        },
        provider_agent_id: query_entry_data.requester_agent_id,
        request_id: query_entry_data.request_id,
        space_address: query_entry_data.space_address,
    };
    Ok(ClientToLib3hResponse::FetchEntryResult(
        fetch_entry_result_data,
    ))
}
