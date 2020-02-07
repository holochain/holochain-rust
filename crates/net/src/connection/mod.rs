//! Provides a lightweight concurrency abstraction for holochain
//! networking / p2p layer

use failure::Error;

pub type NetResult<T> = Result<T, Error>;

pub mod net_connection;
pub mod net_connection_thread;
