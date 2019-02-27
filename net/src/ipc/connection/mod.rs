//! common types and traits for working with Connection instances

mod connection_error;

/// a connection identifier
pub type ConnectionId = String;

pub use self::connection_error::{ConnectionError, ConnectionResult};

/// type name for a bool indicating if work was done during a `poll()`
pub type DidWork = bool;

/// events that can be generated during a connection `poll()`
#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionEvent {
    ConnectionError(ConnectionId, ConnectionError),
    Connect(ConnectionId),
    Message(ConnectionId, Vec<u8>),
    Close(ConnectionId),
}

/// represents a pool of connections to remote nodes
pub trait Connection {
    /// establish a connection to a remote node
    fn connect(&mut self, uri: &str) -> ConnectionResult<ConnectionId>;

    /// close an existing open connection
    fn close(&mut self, id: ConnectionId) -> ConnectionResult<()>;

    /// do some work... this should be called very frequently on an event loop
    fn poll(&mut self) -> ConnectionResult<(DidWork, Vec<ConnectionEvent>)>;

    /// send a payload to remote nodes
    fn send(&mut self, id_list: Vec<ConnectionId>, payload: Vec<u8>) -> ConnectionResult<()>;
}
