use crate::{
    dht::bbdht::dynamodb::client::Client,
    trace::{tracer, LogContext},
};
use lib3h_protocol::data_types::FetchEntryResultData;

// -- Entry -- //
// Another node, or the network module itself is requesting data from us
// all entries are in the database
// no-op
pub fn handle_fetch_entry(
    log_context: &LogContext,
    _client: &Client,
    fetch_entry_data: &FetchEntryResultData,
) {
    tracer(
        &log_context,
        &format!("handle_fetch_entry {:?}", fetch_entry_data),
    );
}
