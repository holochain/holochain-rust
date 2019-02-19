//! The library implementing the holochain pattern of validation rules + local source chain + DHT
#![feature(try_from, arbitrary_self_types, futures_api, async_await, await_macro)]
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
extern crate holochain_core_types_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pretty_assertions;

pub mod action;
pub mod agent;
pub mod context;
pub mod dht;
pub mod instance;
#[cfg(test)]
pub mod link_tests;
pub mod logger;
pub mod network;
pub mod nucleus;
pub mod persister;
pub mod signal;
pub mod state;
pub mod workflows;
