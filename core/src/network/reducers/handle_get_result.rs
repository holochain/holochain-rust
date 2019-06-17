use crate::{
    action::{ActionWrapper},
    network::state::NetworkState,
    state::State,
};

pub fn reduce_handle_get_result(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (maybe_entry, key) = unwrap_to!(action => crate::action::Action::HandleGetResult);

    network_state
        .get_entry_with_meta_results
        .insert(key.clone(), Some(Ok(maybe_entry.clone())));
}
