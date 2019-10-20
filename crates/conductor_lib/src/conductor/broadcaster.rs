use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use jsonrpc_ws_server::ws;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::{io::Write, path::PathBuf};

/// An abstraction which represents the ability to (maybe) send a message to the client
/// over the existing connection.
#[derive(Debug)]
pub enum Broadcaster {
    Ws(ws::Sender),
    #[cfg(unix)]
    UnixSocket(PathBuf),
    Noop,
}

impl Drop for Broadcaster {
    fn drop(&mut self) {
        match self {
            Broadcaster::Ws(sender) => sender.close(ws::CloseCode::Normal).unwrap_or(()),
            Broadcaster::UnixSocket(_) => (), //stream.shutdown(Shutdown::Both).unwrap_or(()),
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
        let msg = msg.into().to_string();
        match self {
            Broadcaster::Ws(sender) => sender.send(ws::Message::Text(msg)).map_err(|e| {
                HolochainError::ErrorGeneric(format!("Broadcaster::Ws -- {}", e.to_string()))
            })?,
            Broadcaster::UnixSocket(path) => {
                let path_str = path.to_str().ok_or("Invalid socket path")?;
                let mut stream = UnixStream::connect(path_str)
                    .map_err(|e| format!("Could not establish Unix domain socket! {:?}", e))?;

                stream.write_all(msg.as_bytes()).map_err(|e| {
                    HolochainError::ErrorGeneric(format!(
                        "Broadcaster::UnixSocket -- {}",
                        e.to_string()
                    ))
                })?;
                // stream.shutdown(Shutdown::);
            }
            Broadcaster::Noop => (),
        }
        Ok(())
    }
}
