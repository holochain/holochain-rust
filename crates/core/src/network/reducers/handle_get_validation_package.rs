use crate::{action::ActionWrapper, network::state::NetworkState, state::State};


#[cfg(not(target_arch = "wasm32"))]
#[flame]
pub fn reduce_handle_get_validation_package(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (address, maybe_validation_package) =
        unwrap_to!(action => crate::action::Action::HandleGetValidationPackage);

    network_state
        .get_validation_package_results
        .insert(address.clone(), Some(Ok(maybe_validation_package.clone())));
}
