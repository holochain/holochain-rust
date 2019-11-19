use crate::{
    action::{Action, ActionWrapper},
    nucleus::{ribosome::MAX_ZOME_CALLS, state::NucleusState, HdkFnCall},
    state::State,
};

/// Reduce AddPendingValidation Action.
/// Inserts boxed EntryWithHeader and dependencies into state, referenced with
/// the entry's address.
#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
pub fn reduce_invoke_hdk_function(
    state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (zome_fn_call, hdk_fn_call) = unwrap_to!(action => Action::InvokeHdkFunction);
    state.zome_call_api_invocations.entry(zome_fn_call).and_modify(|e|{
        let (hdk_fn_call)
    });
}
