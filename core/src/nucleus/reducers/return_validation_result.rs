use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::state::NucleusState,
};
use std::sync::Arc;

pub fn reduce_return_validation_result(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let ((id, hash), validation_result) = unwrap_to!(action => Action::ReturnValidationResult);
    state
        .validation_results
        .insert((id.clone(), hash.clone()), validation_result.clone());
}