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
    let fr = unwrap_to!(action => Action::ReturnZomeFunctionResult);
    // @TODO store the action and result directly
    // @see https://github.com/holochain/holochain-rust/issues/198
    state.zome_calls.insert(fr.call(), Some(fr.result()));
}

/// Reduce SignalZomeFunctionCall Action.
/// Adds the call with a None result so we represent in the state that
/// a zome call is running
pub fn reduce_signal_zome_function(
    state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let call = unwrap_to!(action => Action::SignalZomeFunctionCall);
    state.zome_calls.insert(call.clone(), None);
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

        assert!(state.zome_calls.contains_key(&fr.call()));
    }
}
