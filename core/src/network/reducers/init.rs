use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    network::{handler::create_handler, state::NetworkState},
};
use holochain_net::p2p_network::P2pNetwork;
use holochain_net_connection::{
    json_protocol::{JsonProtocol, TrackDnaData},
    net_connection::NetSend,
};
use std::sync::{Arc, Mutex};

pub fn reduce_init(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let network_settings = unwrap_to!(action => Action::InitNetwork);
    let mut network =
        P2pNetwork::new(create_handler(&context), &network_settings.p2p_config).unwrap();

    // Configure network logger
    // Enable this for debugging network
    //    {
    //        let mut tweetlog = TWEETLOG.write().unwrap();
    //        tweetlog.set(LogLevel::Debug, None);
    //        // set level per tag
    //        tweetlog.set(LogLevel::Debug, Some("memory_server".to_string()));
    //        tweetlog.listen_to_tag("memory_server", Tweetlog::console);
    //        tweetlog.listen(Tweetlog::console);
    //    }

    let json = JsonProtocol::TrackDna(TrackDnaData {
        dna_address: network_settings.dna_address.clone(),
        agent_id: network_settings.agent_id.clone(),
    });

    let _ = network.send(json.into()).and_then(|_| {
        state.network = Some(Arc::new(Mutex::new(network)));
        state.dna_address = Some(network_settings.dna_address.clone());
        state.agent_id = Some(network_settings.agent_id.clone());
        Ok(())
    });
}
