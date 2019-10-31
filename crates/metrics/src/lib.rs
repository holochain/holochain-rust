//! The library implementing the holochain pattern of validation rules + local source chain + DHT
#![feature(arbitrary_self_types, async_await)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate log;

pub mod cloudwatch;
pub mod config;
pub mod metrics;

pub use cloudwatch::*;
pub use config::*;
pub use metrics::*;
