#![feature(try_from)]
#![feature(fnbox)]

//! Provides a lightweight concurrency abstraction for holochain
//! networking / p2p layer
//! see holochain_net_ipc for a specific implementation, and
//! holochain_net for the crate that pulls the implementations together

extern crate byteorder;
#[macro_use]
extern crate failure;
extern crate holochain_core_types;
#[macro_use]
extern crate holochain_core_types_derive;
extern crate rmp;
extern crate rmp_serde;
extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_derive;
// macros only used in tests
#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;

use failure::Error;

pub type NetResult<T> = Result<T, Error>;

pub mod net_connection;
pub mod net_connection_thread;
pub mod net_relay;
pub mod protocol;
pub mod protocol_wrapper;
