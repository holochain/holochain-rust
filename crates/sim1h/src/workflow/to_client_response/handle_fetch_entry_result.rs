use crate::{
    dht::bbdht::{dynamodb::client::Client, error::BbDhtResult},
    trace::{tracer, LogContext},
    workflow::from_client::publish_entry::publish_entry,
};
use lib3h_protocol::data_types::{FetchEntryResultData, ProvidedEntryData};

/// Successful data response for a `HandleFetchEntryData` request
/// result of no-op is no-op
pub fn handle_fetch_entry_result(
    log_context: &LogContext,
    client: &Client,
    fetch_entry_result_data: &FetchEntryResultData,
) -> BbDhtResult<()> {
    tracer(
        &log_context,
        &format!("handle_fetch_entry_result {:?}", fetch_entry_result_data),
    );

    if fetch_entry_result_data.request_id == String::from("fetch-and-publish") {
        publish_entry(
            log_context,
            client,
            &ProvidedEntryData {
                space_address: fetch_entry_result_data.space_address.clone(),
                provider_agent_id: fetch_entry_result_data.provider_agent_id.clone(),
                entry: fetch_entry_result_data.entry.clone(),
            },
        )?;
    }

    Ok(())
}
