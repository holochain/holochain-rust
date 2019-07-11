use crate::{
    action::{Action, ActionWrapper},
    network::state::NetworkState,
    state::State,
};
use holochain_net::connection::{
    json_protocol::{JsonProtocol, TrackDnaData},
    net_connection::NetSend,
};
use std::{thread::sleep, time::Duration};

pub fn reduce_shutdown(
    state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    assert_eq!(*action, Action::ShutdownNetwork);

    let json = JsonProtocol::UntrackDna(TrackDnaData {
        dna_address: state
            .dna_address
            .as_ref()
            .expect("Tried to shutdown uninitialized network")
            .clone(),
        agent_id: state
            .agent_id
            .as_ref()
            .expect("Tried to shutdown uninitialized network")
            .clone()
            .into(),
    });

    let mut network_lock = state.network.lock().unwrap();

    {
        let network = network_lock
            .as_mut()
            .expect("Tried to shutdown uninitialized network");
        let _ = network.send(json.into());
        sleep(Duration::from_secs(2));
    }

    if let Err(err) = network_lock.take().unwrap().stop() {
        println!("ERROR stopping network thread: {:?}", err);
    } else {
        println!("Network thread successfully stopped");
    }
}
