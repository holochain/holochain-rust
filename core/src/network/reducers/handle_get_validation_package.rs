use crate::{action::ActionWrapper, context::Context, network::state::NetworkState};
use std::sync::Arc;

pub fn reduce_handle_get_validation_package(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (address, maybe_validation_package) =
        unwrap_to!(action => crate::action::Action::HandleGetValidationPackage);

    network_state
        .get_validation_package_results
        .insert(address.clone(), Some(Ok(maybe_validation_package.clone())));
}
