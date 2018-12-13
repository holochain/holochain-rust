#![feature(try_from)]
#[macro_use]
extern crate neon;
#[macro_use]
extern crate neon_serde;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate holochain_cas_implementations;
extern crate holochain_container_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_net;
extern crate tempfile;

pub mod app;
mod config;
// mod test_container;
