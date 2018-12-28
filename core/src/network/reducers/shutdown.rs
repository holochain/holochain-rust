use crate::{action::ActionWrapper, context::Context, network::state::NetworkState};
use holochain_net::{p2p_config::P2pConfig, p2p_network::P2pNetwork};
use std::{mem, sync::Arc};

pub fn reduce_shutdown(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    _action_wrapper: &ActionWrapper,
) {
    if let Some(network_mutex) = &network_state.network {
        let mut network = network_mutex.lock().unwrap();
        let mock_network =
            P2pNetwork::new(Box::new(|_r| Ok(())), &P2pConfig::default_mock()).unwrap();
        // hot-swap the real network with a mock network so we can shut down the real one
        mem::replace(&mut *network, mock_network).stop().unwrap();
    }
}
