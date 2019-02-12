use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::state::NucleusState,
};
use std::sync::Arc;

pub fn reduce_return_validation_package(
    _context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (id, maybe_validation_package) = unwrap_to!(action => Action::ReturnValidationPackage);
    state
        .validation_packages
        .insert(id.clone(), maybe_validation_package.clone());
}
