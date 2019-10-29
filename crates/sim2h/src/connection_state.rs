//! represents the state of connected agents
use crate::wire_message::WireMessage;
use lib3h::rrdht_util::Location;
use lib3h_protocol::types::{AgentPubKey, SpaceHash};
pub type AgentId = AgentPubKey;

use crate::error::*;
use lib3h::rrdht_util::*;
use lib3h_crypto_api::CryptoSystem;

#[derive(PartialEq, Debug, Clone)]
pub struct DhtData {
    pub location: Location,
}

impl DhtData {
    /// construct a new DhtData item
    /// will calculate the given agent_id's rrdht location
    #[allow(clippy::borrowed_box)]
    pub fn new(crypto: &Box<dyn CryptoSystem>, agent_id: &AgentId) -> Sim2hResult<Self> {
        Ok(Self {
            location: calc_location_for_id(crypto, &agent_id.to_string())?,
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ConnectionState {
    #[allow(clippy::all)]
    Limbo(Box<Vec<WireMessage>>),
    Joined(SpaceHash, AgentId, DhtData),
}

impl ConnectionState {
    pub fn new() -> ConnectionState {
        ConnectionState::Limbo(Box::new(Vec::new()))
    }

    /// construct a new "Joined" ConnectionState item
    #[allow(clippy::borrowed_box)]
    pub fn new_joined(
        crypto: &Box<dyn CryptoSystem>,
        space_hash: SpaceHash,
        agent_id: AgentId,
    ) -> Sim2hResult<Self> {
        let dht_data = DhtData::new(crypto, &agent_id)?;
        Ok(ConnectionState::Joined(space_hash, agent_id, dht_data))
    }

    pub fn in_limbo(&self) -> bool {
        match self {
            ConnectionState::Limbo(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_connection_state() {
        let ca = ConnectionState::new();
        assert_eq!(ca, ConnectionState::Limbo(Box::new(Vec::new())));
    }
}
