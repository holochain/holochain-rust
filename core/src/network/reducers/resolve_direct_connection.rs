use crate::{
    action::ActionWrapper,
    context::{ContextOnly, ContextStateful},
    network::state::NetworkState,
};
use std::sync::Arc;

pub fn reduce_resolve_direct_connection(
    _context: Arc<ContextStateful>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let id = unwrap_to!(action => crate::action::Action::ResolveDirectConnection);

    network_state.direct_message_connections.remove(id);
}
