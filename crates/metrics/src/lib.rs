//! The library implementing the holochain pattern of validation rules + local source chain + DHT
#![feature(arbitrary_self_types)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate shrinkwraprs;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate holochain_common;

pub mod cloudwatch;
pub mod config;
pub mod logger;
pub mod metrics;
pub mod stats;

pub use cloudwatch::*;
pub use config::*;
pub use metrics::*;
new_relic_setup!("NEW_RELIC_LICENSE_KEY");

pub const CHANNEL_SIZE: usize = 1000;
