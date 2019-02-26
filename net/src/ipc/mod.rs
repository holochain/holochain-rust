//! Networking / P2P IPC Abstraction
//!
//! This crate allows holochain to connect to a running P2P client node
//! over ZeroMq-based socket connection. The recommended ZeroMQ configuration
//! is to use the `ipc:// ` protocol, which will make use of unix domain
//! sockets in a linux or macOs environment. You may need to fall back to
//! `tcp://` for other operating systems.
//!
//! The main export you should care about is ZmqIpcClient.
//!

mod transport;
mod transport_wss;

pub use transport::{DidWork, Transport, TransportError, TransportEvent, TransportResult};

pub use transport_wss::TransportWss;

#[macro_use]
pub mod errors;
pub mod util;

pub mod spawn;
