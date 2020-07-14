#![feature(try_trait)]
#![warn(unused_extern_crates)]
//! Holochain Conductor API
//!
//! This crate is a library that provides types and functions that help with building
//! a Holochain Conductor as described in [ADR15](doc/architecture/decisions/0015-conductor-api).
//!
//! Depending on the specific (application) use-case, the context in which a Holochain instance
//! is run may vary drastically. Application developers may want to bundle Holochain with
//! and statically link the core library into their custom made executable.
//! In such a case, #[holochain.rs](conductor_api/src/holochain.rs) may be used directly as a
//! wrapper around a single instance.
//!
//! In the general case, many different DNAs are being executed alongside each other in the
//! context of the same agent, i.e. user. [conductor.rs](conductor_api/src/conductor.rs) provides
//! a struct that instantiates, holds, manages several Holochain instances.
//! It makes use of [config.rs](conductor_api/src/config.rs) which provides structs for conductor
//! configuration that can be de-/serialized into config files like
//! [these](https://hackmd.io/OcT2cI1ETfu4DHyvn4QZ5A#).
//!
//! All of the above is used in the [conductor crate](conductor).
//!
//! # Example
//! ```rust
//! extern crate holochain_conductor_lib;
//! extern crate holochain_core_types;
//! #[macro_use]
//! extern crate structopt;
//!
//! use holochain_conductor_lib::{
//!     config::{load_configuration, Configuration},
//!     conductor::Conductor,
//! };
//! use holochain_core_types::error::HolochainError;
//! use std::{fs::File, io::prelude::*, path::PathBuf, sync::Arc};
//! use structopt::StructOpt;
//!
//! #[derive(StructOpt, Debug)]
//! #[structopt(name = "hcc")]
//! struct Opt {
//!     /// Path to the toml configuration file for the conductor
//!     #[structopt(short = "c", long = "config", parse(from_os_str))]
//!     config: Option<PathBuf>,
//! }
//!
//!     let opt = Opt::from_args();
//!     let config_path = opt.config
//!         .unwrap_or(PathBuf::from(r"~/.holochain/conductor/conductor_config.toml"));
//!     let config_path_str = config_path.to_str().unwrap();
//!     println!("Using config path: {}", config_path_str);
//!     match bootstrap_from_config(config_path_str) {
//!         Ok(mut conductor) => {
//!             if conductor.instances().len() > 0 {
//!                 println!(
//!                     "Successfully loaded {} instance configurations",
//!                     conductor.instances().len()
//!                 );
//!                 println!("Starting all of them...");
//!                 conductor.start_all_instances();
//!                 println!("Done.");
//!                 loop {}
//!             } else {
//!                 println!("No instance started, bailing...");
//!             }
//!         }
//!         Err(error) => println!("Error while trying to boot from config: {:?}", error),
//!     };
//!
//! fn bootstrap_from_config(path: &str) -> Result<Conductor, HolochainError> {
//!     let config = load_config_file(&String::from(path))?;
//!     config
//!         .check_consistency(&mut Arc::new(Box::new(Conductor::load_dna)))
//!         .map_err(|string| HolochainError::ConfigError(string))?;
//!     let mut conductor = Conductor::from_config(config);
//!     conductor.boot_from_config()?;
//!     Ok(conductor)
//! }
//!
//! fn load_config_file(path: &String) -> Result<Configuration, HolochainError> {
//!     let mut f = File::open(path)?;
//!     let mut contents = String::new();
//!     f.read_to_string(&mut contents)?;
//!     Ok(load_configuration::<Configuration>(&contents)?)
//! }
//! ```
#[macro_use]
extern crate holochain_core;
#[macro_use]
extern crate holochain_json_derive;
#[macro_use]
extern crate holochain_tracing_macros;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[cfg(test)]
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate lazy_static;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate nickel;
#[macro_use]
extern crate holochain_common;

#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod conductor;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod config;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod context_builder;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod dna_location;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod dpki_instance;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod error;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod happ_bundle;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod holo_signing_service;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod holochain;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod interface;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod interface_impls;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod key_loaders;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod keystore;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod logger;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod port_utils;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod signal_wrapper;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod static_file_server;
#[allow(clippy::suspicious_else_formatting, clippy::redundant_closure, clippy::let_and_return, clippy::collapsible_if)]
pub mod static_server_impls;

pub use crate::holochain::Holochain;

new_relic_setup!("NEW_RELIC_LICENSE_KEY");
