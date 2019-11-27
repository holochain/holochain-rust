use crate::{
    action::{Action, ActionWrapper},
    nucleus::{state::NucleusState, ZomeFnCall},
    state::{State, ACTION_PRUNE_MS},
};
use std::time::Duration;

pub fn reduce_prune(
    nucleus_state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    assert_eq!(action_wrapper.action(), &Action::Prune);

    nucleus_state
        .zome_call_results
        .iter()
        .filter_map(|(call, (_result, time))| {
            if let Ok(elapsed) = time.elapsed() {
                if elapsed > Duration::from_millis(ACTION_PRUNE_MS) {
                    return Some(call);
                }
            }
            None
        })
        .cloned()
        .collect::<Vec<ZomeFnCall>>()
        .into_iter()
        .for_each(|action| {
            nucleus_state.zome_call_results.remove(&action);
        });
}
