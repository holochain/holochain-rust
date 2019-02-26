//! common types and traits for working with Transport instances

mod error;

/// a connection identifier
pub type TransportId = String;
pub type TransportIdRef = str;

pub use self::error::{TransportError, TransportResult};

/// type name for a bool indicating if work was done during a `poll()`
pub type DidWork = bool;

/// events that can be generated during a connection `poll()`
#[derive(Debug, PartialEq, Clone)]
pub enum TransportEvent {
    TransportError(TransportId, TransportError),
    Connect(TransportId),
    Message(TransportId, Vec<u8>),
    Close(TransportId),
}

/// represents a pool of connections to remote nodes
pub trait Transport {
    /// establish a connection to a remote node
    fn connect(&mut self, uri: &str) -> TransportResult<TransportId>;

    /// close an existing open connection
    fn close(&mut self, id: TransportId) -> TransportResult<()>;

    /// close all existing open connections
    fn close_all(&mut self) -> TransportResult<()>;

    /// do some work... this should be called very frequently on an event loop
    fn poll(&mut self) -> TransportResult<(DidWork, Vec<TransportEvent>)>;

    /// send a payload to remote nodes
    fn send(&mut self, id_list: &[&TransportIdRef], payload: &[u8]) -> TransportResult<()>;
}
