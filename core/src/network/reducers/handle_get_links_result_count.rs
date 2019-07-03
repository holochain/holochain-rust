use crate::{action::ActionWrapper, network::state::NetworkState, state::State};

pub fn reduce_handle_get_links_count(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (links_count, key) = unwrap_to!(action => crate::action::Action::HandleGetLinksResultCount);
    network_state
        .get_links_results_count
        .insert(key.clone(), Some(Ok(links_count.clone())));
}

pub fn reduce_handle_get_links_count_by_tag(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (links_count, key) =
        unwrap_to!(action => crate::action::Action::HandleGetLinksResultCountByTag);
    network_state
        .get_links_result_count_by_tag
        .insert(key.clone(), Some(Ok(links_count.clone())));
}
