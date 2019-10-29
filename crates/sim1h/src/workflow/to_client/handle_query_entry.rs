use crate::{
    dht::bbdht::dynamodb::client::Client,
    trace::{tracer, LogContext},
};
use lib3h_protocol::data_types::QueryEntryResultData;

// Request a node to handle a QueryEntry request
// queries are simulated on the outgoing side
// no-op
pub fn handle_query_entry(
    log_context: &LogContext,
    _client: &Client,
    query_entry_data: &QueryEntryResultData,
) {
    tracer(
        &log_context,
        &format!("handle_query_entry {:?}", query_entry_data),
    );
}
