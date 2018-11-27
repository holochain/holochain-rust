use holochain_net_connection::protocol_wrapper::TrackAppData;
use crate::network::handler::create_handler;
use crate::action::ActionWrapper;
use crate::network::state::NetworkState;
use crate::context::Context;
use std::sync::Arc;
use std::sync::Mutex;
use crate::action::Action;
use holochain_net::p2p_network::P2pNetwork;
use holochain_net_connection::protocol_wrapper::ProtocolWrapper;
use holochain_net_connection::net_connection::NetConnection;

pub fn reduce_init(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
){
    let action = action_wrapper.action();
    let (_, dna_hash, agent_id) = unwrap_to!(action => Action::InitNetwork);
    let mut network = P2pNetwork::new(
        create_handler(&context),
        &context.network_config
    ).unwrap();

    let _ = network.send(
        ProtocolWrapper::TrackApp(
                TrackAppData{
                    dna_hash: dna_hash.clone(),
                    agent_id: agent_id.clone(),
                })
            .into()
    )
        .and_then(|_| {
            state.network = Some(Arc::new(Mutex::new(network)));
            state.dna_hash = Some(dna_hash.clone());
            state.agent_id = Some(agent_id.clone());
            Ok(())
        });
}
