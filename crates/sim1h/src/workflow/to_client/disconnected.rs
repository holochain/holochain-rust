use crate::{
    dht::bbdht::dynamodb::client::Client,
    trace::{tracer, LogContext},
};
use lib3h_protocol::data_types::DisconnectedData;

// Notification of disconnection from a network
// no-op
pub fn disconnected(
    log_context: &LogContext,
    _client: &Client,
    disconnected_data: &DisconnectedData,
) {
    tracer(
        &log_context,
        &format!("disconnected {:?}", disconnected_data),
    );
}
