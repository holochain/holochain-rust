//! Provides a lightweight concurrency abstraction for holochain
//! networking / p2p layer
//! see holochain_net::ipc for a specific implementation, and
//! holochain_net for the crate that pulls the implementations together

use failure::Error;

pub type NetResult<T> = Result<T, Error>;

pub mod json_protocol;
pub mod net_connection;
pub mod net_connection_thread;
pub mod net_relay;
pub mod protocol;
