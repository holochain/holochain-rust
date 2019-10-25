use crate::dht::bbdht::dynamodb::api::agent::inbox::send_to_agent_inbox;
use crate::dht::bbdht::dynamodb::client::Client;
use crate::dht::bbdht::error::BbDhtResult;
use crate::trace::tracer;
use crate::trace::LogContext;
use lib3h_protocol::data_types::DirectMessageData;
use lib3h_protocol::protocol::ClientToLib3hResponse;

/// A: append message to inbox in database
pub fn send_direct_message(
    log_context: &LogContext,
    client: &Client,
    direct_message_data: &DirectMessageData,
) -> BbDhtResult<ClientToLib3hResponse> {
    tracer(&log_context, "send_direct_message");
    send_to_agent_inbox(
        &log_context,
        &client,
        &direct_message_data.space_address.to_string(),
        &direct_message_data.request_id,
        &direct_message_data.from_agent_id,
        &direct_message_data.to_agent_id,
        &direct_message_data.content,
        false,
    )?;
    Ok(ClientToLib3hResponse::SendDirectMessageResult(
        direct_message_data.clone(),
    ))
}
