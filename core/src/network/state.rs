use holochain_net::p2p_network::{P2pNetwork};
//use holochain_net_connection::{
//    NetResult,
//    net_connection::NetConnection,
//    protocol_wrapper::{
//        DhtData,
//        ProtocolWrapper, //TrackAppData,
//    }
//};
use snowflake;
use std::{
    sync::{Arc, Mutex}
};

#[derive(Clone, Debug)]
pub struct NetworkState {
    pub network: Option<Arc<Mutex<P2pNetwork>>>,
    pub dna_hash: Option<String>,
    pub agent_id: Option<String>,
    id: snowflake::ProcessUniqueId,
}

impl PartialEq for NetworkState {
    fn eq(&self, other: &NetworkState) -> bool {
        self.id == other.id
    }
}

impl NetworkState {
    pub fn new() -> Self {
        NetworkState {
            network: None,
            dna_hash: None,
            agent_id: None,
            id: snowflake::ProcessUniqueId::new(),
        }
    }
}
