//! The library implementing the holochain pattern of validation rules + local source chain + DHT

#![feature(arbitrary_self_types, async_closure, proc_macro_hygiene)]
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

extern crate holochain_wasm_utils;
#[macro_use]
extern crate holochain_json_derive;
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate log;
#[macro_use]
extern crate holochain_logging;
extern crate holochain_tracing as ht;
#[macro_use]
extern crate holochain_tracing_macros;
#[macro_use]
extern crate holochain_common;
extern crate holochain_wasmer_host;
extern crate holochain_wasm_engine;

#[macro_use]
pub mod macros;

#[autotrace]
pub mod action;
#[autotrace]
pub mod agent;
#[autotrace]
pub mod consistency;
#[autotrace]
pub mod content_store;
#[autotrace]
pub mod context;
pub mod dht;
pub mod entry;
#[autotrace]
pub mod instance;
#[cfg(test)]
pub mod link_tests;
pub mod logger;
#[autotrace]
pub mod network;
#[autotrace]
pub mod nucleus;
#[autotrace]
pub mod persister;
pub mod scheduled_jobs;
#[autotrace]
pub mod signal;
#[autotrace]
pub mod state;
#[autotrace]
pub mod state_dump;
pub mod workflows;

new_relic_setup!("NEW_RELIC_LICENSE_KEY");
