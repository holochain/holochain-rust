#![feature(vec_remove_item)]
#![allow(clippy::all)] // As per the request of the networking team

//! holochain_net is a library that defines an abstract networking layer for
//! different network transports, providing a configurable interface
//! for swapping different backends connection modules at load time

#[macro_use]
extern crate failure;
extern crate holochain_common;
#[macro_use]
pub extern crate holochain_json_derive;

extern crate holochain_json_api;
extern crate holochain_persistence_api;

#[macro_use]
extern crate lazy_static;
extern crate lib3h_sodium;
extern crate libc;
extern crate reqwest;
extern crate sha2;
// macros used in tests
#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate logging;

extern crate env_logger;

// wss
extern crate native_tls;
extern crate tungstenite;
extern crate url;

#[macro_use]
pub mod tweetlog;
pub mod connection;
pub mod error;
pub mod ipc;
pub mod ipc_net_worker;
pub mod lib3h_worker;
pub mod sim1h_worker;
pub mod sim2h_worker;
pub mod p2p_config;
pub mod p2p_network;
pub mod in_memory;
