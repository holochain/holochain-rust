use crate::{action::ActionWrapper, network::state::NetworkState, state::State};

pub fn reduce_handle_get_links_result(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (links, key) = unwrap_to!(action => crate::action::Action::HandleGetLinksResult);
    network_state
        .get_links_results
        .insert(key.clone(), Some(Ok(links.clone())));
}
