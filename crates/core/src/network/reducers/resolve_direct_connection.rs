use crate::{action::ActionWrapper, network::state::NetworkState, NEW_RELIC_LICENSE_KEY,state::State};

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_resolve_direct_connection(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let id = unwrap_to!(action => crate::action::Action::ResolveDirectConnection);

    network_state.direct_message_connections.remove(id);
    network_state.direct_message_timeouts.remove(id);
}
