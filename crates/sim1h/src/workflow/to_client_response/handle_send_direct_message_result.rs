use crate::trace::{tracer, LogContext};
use lib3h_protocol::data_types::DirectMessageData;

/// Our response to a direct message from another agent.
/// A sends message to B
/// B told A it received the message
pub fn handle_send_direct_message_result(
    log_context: &LogContext,
    direct_message_data: &DirectMessageData,
) {
    tracer(
        &log_context,
        &format!(
            "handle_send_direct_message_result {:?}",
            direct_message_data
        ),
    );
}
