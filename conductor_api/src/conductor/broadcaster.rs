use holochain_core_types::{error::HolochainError, json::JsonString};
use jsonrpc_ws_server::ws;

/// An abstraction which represents the ability to (maybe) send a message to the client
/// over the existing connection.
#[derive(Debug)]
pub enum Broadcaster {
    Ws(ws::Sender),
    Noop,
}

impl Drop for Broadcaster {
    fn drop(&mut self) {
        match self {
            Broadcaster::Ws(sender) => sender.close(ws::CloseCode::Normal).unwrap_or(()),
            Broadcaster::Noop => (),
        }
    }
}

impl Broadcaster {
    /// Implements the actual method of sending for whichever variant of Broadcaster we have
    pub fn send<J>(&self, msg: J) -> Result<(), HolochainError>
    where
        J: Into<JsonString>,
    {
        match self {
            Broadcaster::Ws(sender) => sender
                .send(ws::Message::Text(msg.into().to_string()))
                .map_err(|e| {
                    HolochainError::ErrorGeneric(format!("Broadcaster::Ws -- {}", e.to_string()))
                })?,
            Broadcaster::Noop => (),
        }
        Ok(())
    }
}
