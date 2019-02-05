//! The library implementing the holochain pattern of validation rules + local source chain + DHT
#![feature(try_from, arbitrary_self_types, futures_api, async_await, await_macro)]
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate futures;
extern crate multihash;
extern crate rust_base58;
extern crate serde;
// serde macro used in tests only
#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;
extern crate snowflake;
#[cfg(test)]
extern crate test_utils;
extern crate wasmi;
#[macro_use]
extern crate unwrap_to;
#[macro_use]
extern crate num_derive;
extern crate num_traits;
extern crate regex;

extern crate config;
extern crate holochain_net;
#[macro_use]
extern crate holochain_wasm_utils;
extern crate holochain_cas_implementations;
extern crate holochain_core_types;
#[macro_use]
extern crate holochain_core_types_derive;
extern crate base64;
extern crate globset;
extern crate holochain_net_connection;
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
