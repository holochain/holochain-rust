use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::NucleusState,
};

pub fn reduce_return_validation_result(
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let ((id, hash), validation_result) = unwrap_to!(action => Action::ReturnValidationResult);
    state
        .validation_results
        .insert((id.clone(), hash.clone()), validation_result.clone());
}
