use crate::{
    action::{Action, ActionWrapper},
    network::state::NetworkState,
    state::State,
    
};

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_clear_query_result(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let query_key = unwrap_to!(action => Action::ClearQueryResult);

    network_state.get_query_results.remove(query_key);
}
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_clear_validation_package_result(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let address = unwrap_to!(action => Action::ClearValidationPackageResult);

    network_state.get_validation_package_results.remove(address);
}
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_clear_custom_send_response(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let id = unwrap_to!(action => Action::ClearCustomSendResponse);

    network_state.custom_direct_message_replys.remove(id);
}
