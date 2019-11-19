use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::{NucleusState, ZomeFnCallState},
    state::State,
};

/// Reduce InvokeHdkFunction Action.
/// Adds unfinished HDK call info to state
pub fn reduce_invoke_hdk_function(
    state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (zome_fn_call, hdk_fn_call) = unwrap_to!(action => Action::InvokeHdkFunction);
    state
        .hdk_function_calls
        .entry(zome_fn_call.clone())
        .and_modify(|zome_fn_call_state| zome_fn_call_state.begin_hdk_call(hdk_fn_call.clone()))
        .or_insert_with(|| ZomeFnCallState::default());
}
