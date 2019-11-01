use crate::{
    action::{Action, ActionWrapper},
    nucleus::state::NucleusState,
    state::State,
};

pub fn reduce_return_validation_package(
    state: &mut NucleusState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (id, maybe_validation_package) = unwrap_to!(action => Action::ReturnValidationPackage);
    state
        .validation_packages
        .insert(id.clone(), maybe_validation_package.clone());
}
