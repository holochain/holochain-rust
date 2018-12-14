use crate::{action::ActionWrapper, context::Context, network::state::NetworkState};
use holochain_core_types::error::HolochainError;
use std::sync::Arc;

pub fn reduce_handle_custom_send_response(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (msg_id, response) = unwrap_to!(action => crate::action::Action::HandleCustomSendResponse);

    network_state.custom_direct_message_replys.insert(
        msg_id.clone(),
        response
            .clone()
            .map_err(|error| HolochainError::ErrorGeneric(error)),
    );
}
