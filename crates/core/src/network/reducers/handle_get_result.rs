use crate::{action::ActionWrapper, network::state::NetworkState, state::State};
//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_handle_get_result(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (payload, key) = unwrap_to!(action => crate::action::Action::HandleQuery);

    network_state
        .get_query_results
        .insert(key.clone(), Some(Ok(payload.clone())));

    network_state.query_timeouts.remove(key);
}
