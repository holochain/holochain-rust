use crate::{
    action::ActionWrapper,
    network::{reducers::send, state::NetworkState},
    state::State,
};
use lib3h_protocol::protocol_client::Lib3hClientProtocol;

pub fn reduce_respond_authoring_list(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let entry_list_data = unwrap_to!(action => crate::action::Action::RespondAuthoringList);
    if let Err(err) = send(
        network_state,
        Lib3hClientProtocol::HandleGetAuthoringEntryListResult(entry_list_data.clone()),
    ) {
        println!(
            "Error sending JsonProtocol::HandleGetAuthoringEntryListResult: {:?}",
            err
        )
    }
}
