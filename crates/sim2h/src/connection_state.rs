//! represents the state of connected agents
use crate::wire_message::WireMessage;
use lib3h::rrdht_util::Location;
use lib3h_protocol::types::{AgentPubKey, SpaceHash};
pub type AgentId = AgentPubKey;

#[derive(PartialEq, Debug, Clone)]
pub struct DhtData {
    pub location: Location,
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
