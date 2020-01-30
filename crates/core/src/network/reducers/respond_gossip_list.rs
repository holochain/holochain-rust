use crate::{
    action::ActionWrapper,
    network::{reducers::send, state::NetworkState},
    state::State,NEW_RELIC_LICENSE_KEY
};
use lib3h_protocol::protocol_client::Lib3hClientProtocol;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_respond_gossip_list(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let entry_list_data = unwrap_to!(action => crate::action::Action::RespondGossipList);
    if let Err(err) = send(
        network_state,
        Lib3hClientProtocol::HandleGetGossipingEntryListResult(entry_list_data.clone()),
    ) {
        println!(
            "Error sending Lib3hClientProtocol::HandleGetGossipEntryListResult: {:?}",
            err
        )
    }
}
