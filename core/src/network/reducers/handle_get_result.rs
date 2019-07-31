use crate::{action::ActionWrapper, network::state::NetworkState, state::State};

pub fn reduce_handle_get_result(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (payload, key) = unwrap_to!(action => crate::action::Action::HandleGet);
    //let maybe_entry = unwrap_to!(payload=>crate::action::RespondGetPayload::Entry);

    network_state
        .get_results
        .insert(key.clone(), Some(Ok(payload.clone())));
}
