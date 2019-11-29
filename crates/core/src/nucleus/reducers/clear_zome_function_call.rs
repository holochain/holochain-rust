use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::NucleusState,
    state::State,
};

pub fn reduce_clear_zome_function_call(
    nucleus_state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let call = unwrap_to!(action => Action::ClearZomeFunctionCall);;

    nucleus_state.queued_zome_calls = nucleus_state
        .queued_zome_calls
        .iter()
        .filter(|c| *c != call)
        .cloned()
        .collect();
    nucleus_state.running_zome_calls.remove(&call);
    nucleus_state.hdk_function_calls.remove(&call);
    nucleus_state.zome_call_results.remove(&call);
}
