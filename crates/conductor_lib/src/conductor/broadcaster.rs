use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use jsonrpc_ws_server::ws;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::{io::Write, net::Shutdown};

/// An abstraction which represents the ability to (maybe) send a message to the client
/// over the existing connection.
#[derive(Debug)]
pub enum Broadcaster {
    Ws(ws::Sender),
    #[cfg(unix)]
    UnixSocket(UnixStream),
    Noop,
}

impl Drop for Broadcaster {
    fn drop(&mut self) {
        match self {
            Broadcaster::Ws(sender) => sender.close(ws::CloseCode::Normal).unwrap_or(()),
            Broadcaster::UnixSocket(stream) => stream.shutdown(Shutdown::Both).unwrap_or(()),
            Broadcaster::Noop => (),
        }
    }
}

impl Broadcaster {
    /// Implements the actual method of sending for whichever variant of Broadcaster we have
    pub fn send<J>(&mut self, msg: J) -> Result<(), HolochainError>
    where
        J: Into<JsonString>,
    {
        match self {
            Broadcaster::Ws(sender) => sender
                .send(ws::Message::Text(msg.into().to_string()))
                .map_err(|e| {
                    HolochainError::ErrorGeneric(format!("Broadcaster::Ws -- {}", e.to_string()))
                })?,
            Broadcaster::UnixSocket(stream) => stream
                .write_all(msg.into().to_string().as_bytes())
                .map_err(|e| {
                    HolochainError::ErrorGeneric(format!(
                        "Broadcaster::UnixSocket -- {}",
                        e.to_string()
                    ))
                })?,
            Broadcaster::Noop => (),
        }
        Ok(())
    }
}
