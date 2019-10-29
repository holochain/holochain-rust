use crate::{
    dht::bbdht::dynamodb::client::Client,
    trace::{tracer, LogContext},
};
use lib3h_protocol::data_types::DirectMessageData;

/// Request to handle a direct message another agent has sent us.
/// A has put something in inbox for B
/// B needs to query to find it and pass to core
pub fn handle_send_direct_message(
    log_context: &LogContext,
    _client: &Client,
    direct_message_data: &DirectMessageData,
) {
    tracer(
        &log_context,
        &format!("handle_send_direct_message {:?}", direct_message_data),
    );
}
