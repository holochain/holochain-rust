//! The library implementing the holochain pattern of validation rules + local source chain + DHT
#![feature(arbitrary_self_types, async_await, async_closure)]
#![warn(unused_extern_crates)]
#[macro_use]
extern crate serde_derive;
// serde macro used in tests only
#[allow(unused_imports)]
#[macro_use]
#[cfg(test)]
extern crate test_utils;
#[macro_use]
extern crate unwrap_to;
#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate holochain_wasm_utils;
#[macro_use]
extern crate holochain_json_derive;
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate log;
#[macro_use]
extern crate holochain_logging;

#[macro_use]
pub mod macros;
pub mod action;
pub mod agent;
pub mod consistency;
pub mod content_store;
pub mod context;
pub mod dht;
pub mod entry;
pub mod instance;
#[cfg(test)]
pub mod link_tests;
pub mod logger;
pub mod network;
pub mod nucleus;
pub mod persister;
pub mod scheduled_jobs;
pub mod signal;
pub mod state;
pub mod state_dump;
pub mod workflows;
