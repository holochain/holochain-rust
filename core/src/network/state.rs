//use base64;
//use context::Context;
use holochain_core_types::{
    cas::content::AddressableContent,
    entry::Entry,
    //error::HolochainError
};
use holochain_net::p2p_network::{P2pNetwork};
use holochain_net_connection::{
    NetResult,
    net_connection::NetConnection,
    protocol_wrapper::{
        DhtData,
        ProtocolWrapper, //TrackAppData,
    }
};
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


    pub fn publish(&self, entry: Entry) -> NetResult<()> {
        if self.network.is_none() || self.dna_hash.is_none() ||  self.agent_id.is_none() {
            bail!("Network not initialized");
        }

        let data = DhtData {
            msg_id: "?".to_string(),
            dna_hash: self.dna_hash.clone().unwrap(),
            agent_id: self.agent_id.clone().unwrap(),
            address: entry.address().to_string(),
            content: serde_json::from_str(&entry.content().to_string())?,
        };

        match self.network {
            None => unreachable!(),
            Some(ref network) => {
                network.lock()
                    .unwrap()
                    .send(ProtocolWrapper::PublishDht(data).into())
            }
        }
    }

}