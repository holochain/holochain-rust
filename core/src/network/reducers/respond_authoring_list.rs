use crate::{
    action::ActionWrapper,
    network::{reducers::send, state::NetworkState},
    state::State,
};
use holochain_net::connection::json_protocol::JsonProtocol;

pub fn reduce_respond_authoring_list(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let entry_list_data = unwrap_to!(action => crate::action::Action::RespondAuthoringList);
    if let Err(err) = send(
        network_state,
        JsonProtocol::HandleGetAuthoringEntryListResult(entry_list_data.clone()),
    ) {
        println!(
            "Error sending JsonProtocol::HandleGetAuthoringEntryListResult: {:?}",
            err
        )
    }
}
