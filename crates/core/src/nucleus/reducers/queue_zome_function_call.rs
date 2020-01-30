use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::NucleusState,
    state::State,
    wasm_engine::MAX_ZOME_CALLS,
    NEW_RELIC_LICENSE_KEY,
};

/// Reduce AddPendingValidation Action.
/// Inserts boxed EntryWithHeader and dependencies into state, referenced with
/// the entry's address.
#[allow(unknown_lints)]
#[allow(clippy::needless_pass_by_value)]
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_queue_zome_function_call(
    state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let call = unwrap_to!(action => Action::QueueZomeFunctionCall);
    if state.running_zome_calls.len() < MAX_ZOME_CALLS {
        state.running_zome_calls.insert(call.clone());
    } else {
        state.queued_zome_calls.push_back(call.clone());
    }
}
