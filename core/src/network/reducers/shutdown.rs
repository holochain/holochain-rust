use crate::{action::ActionWrapper, context::Context, network::state::NetworkState};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{ProtocolWrapper, TrackAppData},
};
use std::sync::Arc;
pub fn reduce_shutdown(
    _context: Arc<Context>,
    state: &mut NetworkState,
    _action_wrapper: &ActionWrapper,
) {
    match (&state.network, &state.dna_address, &state.agent_id) {
        (Some(network), Some(dna_address), Some(agent_id)) => {
            network
                .lock()
                .unwrap()
                .send(
                    ProtocolWrapper::DropApp(TrackAppData {
                        dna_address: dna_address.to_owned(),
                        agent_id: agent_id.to_string(),
                    })
                    .into(),
                )
                .and_then(|_| Ok(()))
                .unwrap_or(());
        }
        _ => {
            // network was never initialized
        }
    }
}
