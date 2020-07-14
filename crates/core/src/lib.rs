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
extern crate holochain_tracing as ht;
#[macro_use]
extern crate holochain_tracing_macros;
#[macro_use]
extern crate holochain_common;

#[macro_use]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod macros;

// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod action;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod agent;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod consistency;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod content_store;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod context;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod dht;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod entry;
#[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod instance;
#[cfg(test)]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod link_tests;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod logger;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod network;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod nucleus;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod persister;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod scheduled_jobs;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod signal;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod state;
// #[autotrace]
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod state_dump;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod wasm_engine;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure)]
pub mod workflows;

new_relic_setup!("NEW_RELIC_LICENSE_KEY");
