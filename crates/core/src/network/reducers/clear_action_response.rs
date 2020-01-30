use crate::{
    action::{Action, ActionWrapper},
    network::state::NetworkState,
    state::State,NEW_RELIC_LICENSE_KEY
};

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_clear_action_response(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let id = unwrap_to!(action => Action::ClearActionResponse);

    network_state.actions = network_state
        .actions
        .iter()
        .filter(|(action, _)| action.id() != id)
        .cloned()
        .collect();
}
