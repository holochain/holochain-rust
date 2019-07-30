use crate::{action::ActionWrapper, network::state::NetworkState, state::State};

pub fn reduce_handle_get_links_result(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (payload, key) = unwrap_to!(action => crate::action::Action::HandleGet);
    let key = unwrap_to!(key=>crate::action::Key::Links);
    let (links,_,_) = unwrap_to!(payload=>crate::action::RespondGetPayload::Links);

    network_state
        .get_links_results
        .insert(key.clone(), Some(Ok(links.clone())));
}
