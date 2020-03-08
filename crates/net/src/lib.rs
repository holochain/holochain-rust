#![feature(vec_remove_item)]
#![allow(clippy::all)] // As per the request of the networking team

//! holochain_net is a library that defines an abstract networking layer for
//! different network transports, providing a configurable interface
//! for swapping different backends connection modules at load time

#[macro_use]
extern crate failure;
//#[macro_use]
//extern crate holochain_common;
#[macro_use]
pub extern crate holochain_json_derive;
extern crate holochain_tracing as ht;
#[macro_use]
extern crate lazy_static;
// macros used in tests
#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

//#[macro_use]
//extern crate holochain_tracing_macros;
pub mod connection;
pub mod error;
pub mod in_memory;
pub mod lib3h_worker;
pub mod p2p_config;
pub mod p2p_network;
pub mod sim2h_worker;
pub mod tweetlog;

//new_relic_setup!("NEW_RELIC_LICENSE_KEY");
