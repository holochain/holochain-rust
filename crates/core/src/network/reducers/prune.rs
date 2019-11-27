use crate::{
    action::{Action, ActionWrapper},
    network::state::NetworkState,
    state::{State, ACTION_PRUNE_MS},
};
use std::time::Duration;

pub fn reduce_prune(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    assert_eq!(action_wrapper.action(), &Action::Prune);

    network_state
        .actions
        .iter()
        .filter_map(|(action, response)| {
            if let Ok(elapsed) = response.created_at.elapsed() {
                if elapsed > Duration::from_millis(ACTION_PRUNE_MS) {
                    return Some(action);
                }
            }
            None
        })
        .cloned()
        .collect::<Vec<ActionWrapper>>()
        .into_iter()
        .for_each(|action| {
            network_state.actions.remove(&action);
        });
}
