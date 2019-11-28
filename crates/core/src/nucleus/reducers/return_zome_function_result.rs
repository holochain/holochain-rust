use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::NucleusState,
    state::State,
};

/// Reduce ReturnZomeFunctionResult Action.
/// Simply drops function call into zome_calls state.
pub fn reduce_return_zome_function_result(
    state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let zome_fn_response = unwrap_to!(action => Action::ReturnZomeFunctionResult);
    state.zome_call_results.insert(
        zome_fn_response.call(),
        zome_fn_response.result(),
    );
    state.running_zome_calls.remove(&zome_fn_response.call());
    state.hdk_function_calls.remove(&zome_fn_response.call());
    if let Some(next_call) = state.queued_zome_calls.pop_front() {
        state.running_zome_calls.insert(next_call);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        action::tests::test_action_wrapper_rzfr, instance::tests::test_context,
        nucleus::state::tests::test_nucleus_state, state::test_store,
    };

    #[test]
    /// test for returning zome function result actions
    fn test_reduce_return_zome_function_result() {
        let context = test_context("jimmy", None);
        let mut state = test_nucleus_state();
        let root_state = test_store(context);
        let action_wrapper = test_action_wrapper_rzfr();

        // @TODO don't juggle action wrappers to get at action in state
        // @see https://github.com/holochain/holochain-rust/issues/198
        let action = action_wrapper.action();
        let fr = unwrap_to!(action => Action::ReturnZomeFunctionResult);

        reduce_return_zome_function_result(&mut state, &root_state, &action_wrapper);

        assert!(state.zome_call_results.contains_key(&fr.call()));
    }
}
