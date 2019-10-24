use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use jsonrpc_ws_server::ws;

/// An abstraction which represents the ability to (maybe) send a message to the client
/// over the existing connection.
pub enum Broadcaster {
    Ws(jsonrpc_ws_server::Broadcaster),
    Noop,
}

impl std::fmt::Debug for Broadcaster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant = match self {
            Broadcaster::Ws(_) => "Ws",
            Broadcaster::Noop => "Noop",
        };
        write!(f, "Broadcaster::{}", variant)
    }
}

impl Broadcaster {
    /// Implements the actual method of sending for whichever variant of Broadcaster we have
    pub fn send<J>(&self, msg: J) -> Result<(), HolochainError>
    where
        J: Into<JsonString>,
    {
        match self {
            Broadcaster::Ws(broadcaster) => broadcaster
                .send(ws::Message::Text(msg.into().to_string()))
                .map_err(|e| {
                    HolochainError::ErrorGeneric(format!("Broadcaster::Ws -- {}", e.to_string()))
                })?,
            Broadcaster::Noop => (),
        }
        Ok(())
    }
}
