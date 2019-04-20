use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    network::{handler::create_handler, state::NetworkState},
};
use holochain_net::{
    connection::{
        json_protocol::{JsonProtocol, TrackDnaData},
        net_connection::NetSend,
    },
    p2p_network::P2pNetwork,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::{Condvar, Mutex};

const P2P_READY_TIMEOUT_MS: u64 = 5000;

pub fn reduce_init(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let network_settings = unwrap_to!(action => Action::InitNetwork);
    let ready_cond_var = Arc::new((Mutex::new(false), Condvar::new()));
    let mut network = P2pNetwork::new(
        create_handler(
            &context,
            network_settings.dna_address.to_string(),
            &ready_cond_var,
        ),
        &network_settings.p2p_config,
    )
    .unwrap();

    let &(ref ready_lock, ref ready_cond_var) = &*ready_cond_var;
    let mut ready = ready_lock.lock();
    let ready_result = ready_cond_var.wait_until(
        &mut ready,
        Instant::now() + Duration::from_millis(P2P_READY_TIMEOUT_MS),
    );

    if ready_result.timed_out() {
        context.log("error/network/reducers: timed out waiting for p2p ready.");
        panic!("p2p networking failed- timed out waiting for p2p ready.");
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
