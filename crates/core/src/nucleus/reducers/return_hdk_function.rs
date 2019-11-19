use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::NucleusState,
    state::State,
};

/// Reduce AddPendingValidation Action.
/// Inserts boxed EntryWithHeader and dependencies into state, referenced with
/// the entry's address.
#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
pub fn reduce_return_hdk_function(
    state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (zome_fn_call, hdk_fn_call, hdk_fn_call_result) =
        unwrap_to!(action => Action::ReturnHdkFunction);
    state
        .zome_call_api_invocations
        .get_mut(zome_fn_call)
        .ok_or_else(|| format!("Cannot record hdk function return for zome call, because its invocation was never recorded. zome call = {:?}, hdk call = {:?}", zome_fn_call, hdk_fn_call))
        .and_then(|zfcs| zfcs.end_hdk_call(hdk_fn_call.clone(), hdk_fn_call_result.clone()).map_err(|e| e.to_string()))
        .unwrap_or_else(|err| error!("{}", err));
}
