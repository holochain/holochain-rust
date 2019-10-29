use crate::{
    dht::bbdht::{
        dynamodb::{api::agent::inbox::send_to_agent_inbox, client::Client},
        error::BbDhtResult,
    },
    trace::{tracer, LogContext},
};
use lib3h_protocol::data_types::DirectMessageData;

// -- Direct Messaging -- //
// the response received from a previous `SendDirectMessage`
// B puts a message back to A
// works exactly the same as the original send
pub fn send_direct_message_result(
    log_context: &LogContext,
    client: &Client,
    direct_message_data: &DirectMessageData,
) -> BbDhtResult<()> {
    tracer(
        &log_context,
        &format!("send_direct_message_result {:?}", direct_message_data),
    );
    send_to_agent_inbox(
        &log_context,
        &client,
        &direct_message_data.space_address.to_string(),
        &direct_message_data.request_id,
        &direct_message_data.from_agent_id,
        &direct_message_data.to_agent_id,
        &direct_message_data.content,
        true,
    )?;
    Ok(())
}
