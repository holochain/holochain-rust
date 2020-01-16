use crate::{action::ActionWrapper, network::state::NetworkState, state::State};
use holochain_core_types::error::HolochainError;

#[autotrace]
pub fn reduce_handle_custom_send_response(
    network_state: &mut NetworkState,
    _root_state: &State,
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
