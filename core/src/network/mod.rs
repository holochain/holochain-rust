use base64;
use context::Context;
use holochain_core_types::error::HolochainError;
use holochain_net::p2p_network::{P2pNetwork};
use holochain_net_connection::{
    net_connection::{NetHandler, NetConnection},
    protocol_wrapper::{
        ProtocolWrapper, TrackAppData,
    }
};
use std::sync::Arc;

struct Network {

}

impl Network {
    pub fn new() -> Self {
        Network {}
    }

    pub fn start(&self, context: Arc<Context>) -> Result<(), HolochainError>{
        let state = context.state()
            .ok_or("Network::start() could not get application state".to_string())?;
        let agent = state.agent().get_agent(&context)?;
        let agent_id = agent.key;

        let dna = state.nucleus().dna().ok_or("Network::start() called without DNA".to_string())?;
        let dna_hash = base64::encode(&dna.multihash()?);

        let mut network = context.network.lock().unwrap();
        (*network).send(ProtocolWrapper::TrackApp(TrackAppData{
            dna_hash,
            agent_id,
        }).into()).map_err(|error| HolochainError::ErrorGeneric(error.to_string()))
    }
}