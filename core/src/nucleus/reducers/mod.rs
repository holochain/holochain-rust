mod add_pending_validation;
pub mod init_application;
pub mod return_initialization_result;
pub mod return_validation_package;
pub mod return_validation_result;
pub mod return_zome_function_result;

use crate::{
    action::{Action, ActionWrapper, NucleusReduceFn},
    context::Context,
    nucleus::{
        reducers::{
            add_pending_validation::reduce_add_pending_validation,
            init_application::reduce_initialize_chain,
            return_initialization_result::reduce_return_initialization_result,
            return_validation_package::reduce_return_validation_package,
            return_validation_result::reduce_return_validation_result,
            return_zome_function_result::reduce_return_zome_function_result,
        },
        state::NucleusState,
    },
};

use std::sync::Arc;

/// Maps incoming action to the correct reducer
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<NucleusReduceFn> {
    match action_wrapper.action() {
        Action::AddPendingValidation(_) => Some(reduce_add_pending_validation),
        Action::ReturnInitializationResult(_) => Some(reduce_return_initialization_result),
        Action::InitializeChain(_) => Some(reduce_initialize_chain),
        Action::ReturnZomeFunctionResult(_) => Some(reduce_return_zome_function_result),
        Action::ReturnValidationResult(_) => Some(reduce_return_validation_result),
        Action::ReturnValidationPackage(_) => Some(reduce_return_validation_package),
        _ => None,
    }
}

/// Reduce state of Nucleus according to action.
/// Note: Can't block when dispatching action here because we are inside the reduce's mutex
pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<NucleusState>,
    action_wrapper: &ActionWrapper,
) -> Arc<NucleusState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: NucleusState = (*old_state).clone();
            f(context, &mut new_state, &action_wrapper);
            Arc::new(new_state)
        }
        None => old_state,
    }
}
