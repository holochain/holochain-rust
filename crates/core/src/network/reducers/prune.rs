use crate::{
    action::{Action, ActionWrapper},
    network::state::NetworkState,
    state::{State, ACTION_PRUNE_MS},
};
use std::time::Duration;

[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_prune(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    assert_eq!(action_wrapper.action(), &Action::Prune);

    network_state.actions = network_state
        .actions
        .iter()
        .filter(|(_, response)| {
            if let Ok(elapsed) = response.created_at.elapsed() {
                if elapsed > Duration::from_millis(ACTION_PRUNE_MS) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();
}
