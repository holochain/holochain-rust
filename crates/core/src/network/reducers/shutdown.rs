use crate::{
    action::{Action, ActionWrapper},
    network::state::NetworkState,
    state::State,
};

use holochain_net::connection::net_connection::NetSend;

use lib3h_protocol::{data_types::SpaceData, protocol_client::Lib3hClientProtocol};
use log::error;
use std::{thread::sleep, time::Duration};

[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_shutdown(
    state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    assert_eq!(*action, Action::ShutdownNetwork);

    let json = Lib3hClientProtocol::LeaveSpace(SpaceData {
        request_id: snowflake::ProcessUniqueId::new().to_string(),
        space_address: state
            .dna_address
            .as_ref()
            .expect("Tried to shutdown uninitialized network")
            .clone()
            .into(),
        agent_id: state
            .agent_id
            .as_ref()
            .expect("Tried to shutdown uninitialized network")
            .clone()
            .into(),
    });

    if let Some(mut network) = state.network.take() {
        let _ = network.send(json);

        sleep(Duration::from_secs(2));

        network.stop();
    } else {
        error!("Tried to shutdown uninitialized network");
    }
}
