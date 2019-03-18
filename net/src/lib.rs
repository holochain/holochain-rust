#![feature(fnbox)]
#![feature(try_from)]
#![feature(vec_remove_item)]

//! holochain_net is a library that defines an abstract networking layer for
//! different network transports, providing a configurable interface
//! for swapping different backends connection modules at load time

#[macro_use]
extern crate failure;
#[macro_use]
pub extern crate holochain_core_types_derive;
#[macro_use]
extern crate lazy_static;
extern crate directories;
extern crate reqwest;
extern crate sha2;
// macros used in tests
#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

// wss
extern crate native_tls;
extern crate tungstenite;
extern crate url;

#[macro_use]
pub mod tweetlog;
pub mod connection;
pub mod error;
pub mod in_memory;
pub mod ipc;
pub mod ipc_net_worker;
pub mod p2p_config;
pub mod p2p_network;
