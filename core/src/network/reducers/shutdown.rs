use crate::{action::ActionWrapper, context::Context, network::state::NetworkState};
use std::sync::Arc;

pub fn reduce_shutdown(
    _context: Arc<Context>,
    _network_state: &mut NetworkState,
    _action_wrapper: &ActionWrapper,
) {
    // @TODO: handle each instance as it shuts down?
    // if so, need to include instance ID in the action, which may be a more general change
}
