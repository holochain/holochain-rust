use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    network::{handler::create_handler, state::NetworkState, },
};
use holochain_net::{
    connection::{
        protocol::Protocol,
        json_protocol::{JsonProtocol, TrackDnaData},
        net_connection::NetSend,
    },
    p2p_network::P2pNetwork,
};
use std::{
    sync::{Arc, mpsc::channel},
    time::Duration,
};

//use parking_lot::{Condvar, Mutex};

const P2P_READY_TIMEOUT_MS: u64 = 10000;

pub fn reduce_init(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let network_settings = unwrap_to!(action => Action::InitNetwork);
    let (sender, receiver) = channel();
    let mut network = P2pNetwork::new(
        create_handler(
            &context,
            network_settings.dna_address.to_string(),
            &sender
        ),
        &network_settings.p2p_config,
    )
    .unwrap();

    let maybe_message =
        receiver.recv_timeout(Duration::from_millis(P2P_READY_TIMEOUT_MS));
    match maybe_message {
        Ok(Protocol::P2pReady) =>
            { context.log("debug/network/reducers: p2p networking ready") },
        Ok(message) =>
        {
            context.log(format!("warn/network/reducers: unexpected \
                         protocol message {:?}", message));
        },
        Err(e) =>
        {
            context.log(format!("err/network/reducers: timed out waiting for p2p \
                        network to be ready: {:?}", e));
            panic!("p2p network not ready within alloted time of {:?} ms",
                  P2P_READY_TIMEOUT_MS);
        }
    }

    // Configure network logger
    // Enable this for debugging network
    //    {
    //        let mut tweetlog = TWEETLOG.write().unwrap();
    //        tweetlog.set(LogLevel::Debug, None);
    //        // set level per tag
    //        tweetlog.set(LogLevel::Debug, Some("memory_server".to_string()));
    //        tweetlog.listen_to_tag("memory_server", Tweetlog::console);
    //        tweetlog.listen(Tweetlog::console);
    //        tweetlog.i("TWEETLOG ENABLED");
    //    }

    let json = JsonProtocol::TrackDna(TrackDnaData {
        dna_address: network_settings.dna_address.clone(),
        agent_id: network_settings.agent_id.clone(),
    });

    let _ = network.send(json.into()).and_then(|_| {
        state.network = Some(Arc::new(std::sync::Mutex::new(network)));
        state.dna_address = Some(network_settings.dna_address.clone());
        state.agent_id = Some(network_settings.agent_id.clone());
        Ok(())
    });
}
