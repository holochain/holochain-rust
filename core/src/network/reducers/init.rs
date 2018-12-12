use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    network::{handler::create_handler, state::NetworkState},
};
use holochain_net::p2p_network::P2pNetwork;
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{ProtocolWrapper, TrackAppData},
};
use std::sync::{Arc, Mutex};

pub fn reduce_init(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let network_settings = unwrap_to!(action => Action::InitNetwork);
    let mut network = P2pNetwork::new(create_handler(&context), &network_settings.config).unwrap();

    let _ = network
        .send(
            ProtocolWrapper::TrackApp(TrackAppData {
                dna_hash: network_settings.dna_hash.clone(),
                agent_id: network_settings.agent_id.clone(),
            })
            .into(),
        )
        .and_then(|_| {
            state.network = Some(Arc::new(Mutex::new(network)));
            state.dna_hash = Some(network_settings.dna_hash.clone());
            state.agent_id = Some(network_settings.agent_id.clone());
            Ok(())
        });
}
