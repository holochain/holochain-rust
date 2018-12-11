use crate::{action::ActionWrapper, context::Context, network::state::NetworkState};
use std::sync::Arc;

pub fn reduce_handle_custom_send_response(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (msg_id, response) =
        unwrap_to!(action => crate::action::Action::HandleCustomSendResponse);

    println!("reducer: {:?} {:?}", msg_id, response);

    network_state
        .custom_direct_message_replys
        .insert(msg_id.clone(), response.clone());
}
