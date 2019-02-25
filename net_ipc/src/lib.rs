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
#![warn(unused_extern_crates)]
#[macro_use]
extern crate failure;
extern crate holochain_net_connection;
#[macro_use]
extern crate lazy_static;
extern crate snowflake;
extern crate zmq;

// wss
extern crate tungstenite;
extern crate url;
extern crate native_tls;

mod connection;
mod connection_wss;

pub use connection::{
    ConnectionError,
    ConnectionResult,
    DidWork,
    ConnectionEvent,
    Connection};

pub use connection_wss::ConnectionWss;

#[macro_use]
pub mod errors;
pub mod context;
pub mod socket;
pub mod util;

pub mod ipc_client;
pub mod spawn;
