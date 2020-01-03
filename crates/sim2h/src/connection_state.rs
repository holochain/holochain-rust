//! represents the state of connected agents
use crate::wire_message::WireMessage;
use lib3h_protocol::types::{AgentPubKey, SpaceHash};
use nanoid;
pub type AgentId = AgentPubKey;

use crate::error::*;

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derive(PartialEq, Debug, Clone)]
pub struct ConnectionStateUniq {
    nid: String,

    #[shrinkwrap(main_field)]
    pub state: ConnectionState,
}

impl ConnectionStateUniq {
    pub fn id(&self) -> &str {
        &self.nid
    }

    pub fn new() -> ConnectionStateUniq {
        ConnectionStateUniq {
            nid: nanoid::simple(),
            state: ConnectionState::new(),
        }
    }

    #[allow(clippy::borrowed_box)]
    pub fn new_joined(space_hash: SpaceHash, agent_id: AgentId) -> Sim2hResult<Self> {
        Ok(Self {
            nid: nanoid::simple(),
            state: ConnectionState::Joined(space_hash, agent_id),
        })
    }
}

// impl From<ConnectionState> for ConnectionStateUniq {
//     fn from(state: ConnectionState) -> ConnectionStateUniq {
//         ConnectionStateUniq {
//             nid: nanoid::simple(),
//             state
//         }
//     }
// }

#[derive(PartialEq, Debug, Clone)]
pub enum ConnectionState {
    #[allow(clippy::all)]
    Limbo(Box<Vec<WireMessage>>),
    Joined(SpaceHash, AgentId),
}

impl ConnectionState {
    pub fn new() -> ConnectionState {
        ConnectionState::Limbo(Box::new(Vec::new()))
    }

    /// construct a new "Joined" ConnectionState item
    #[allow(clippy::borrowed_box)]
    pub fn new_joined(space_hash: SpaceHash, agent_id: AgentId) -> Sim2hResult<Self> {
        Ok(ConnectionState::Joined(space_hash, agent_id))
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
