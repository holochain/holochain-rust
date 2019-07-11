use crate::{
    action::ActionWrapper,
    network::{reducers::send, state::NetworkState},
    state::State,
};
use holochain_net::connection::json_protocol::JsonProtocol;

pub fn reduce_respond_gossip_list(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let entry_list_data = unwrap_to!(action => crate::action::Action::RespondGossipList);
    if let Err(err) = send(
        network_state,
        JsonProtocol::HandleGetGossipingEntryListResult(entry_list_data.clone()),
    ) {
        println!(
            "Error sending JsonProtocol::HandleGetGossipEntryListResult: {:?}",
            err
        )
    }
}
