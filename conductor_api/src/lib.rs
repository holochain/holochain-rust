//! let file_system = Arc::new(RwLock::new(FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap()));
//!     Arc::new(Mutex::new(SimplePersister::new(file_system.clone()))),
//!     file_system.clone(),

#![feature(try_from, try_trait, async_await, await_macro)]
#![warn(unused_extern_crates)]
/// Holochain Conductor API
///
/// This crate is a library that provides types and functions that help with building
/// a Holochain Conductor as described in [ADR15](doc/architecture/decisions/0015-conductor-api).
///
/// Depending on the specific (application) use-case, the context in which a Holochain instance
/// is run may vary drastically. Application developers may want to bundle Holochain with
/// and statically link the core library into their custom made executable.
/// In such a case, [holochain.rs](conductor_api/src/holochain.rs) may be used directly as a
/// wrapper around a single instance.
///
/// In the general case, many different DNAs are being executed alongside each other in the
/// context of the same agent, i.e. user. [conductor.rs](conductor_api/src/conductor.rs) provides
/// a struct that instantiates, holds, manages several Holochain instances.
/// It makes use of [config.rs](conductor_api/src/config.rs) which provides structs for conductor
/// configuration that can be de-/serialized into config files like
/// [these](https://hackmd.io/OcT2cI1ETfu4DHyvn4QZ5A#).
///
/// All of the above is used in the [conductor crate](conductor).
///
/// # Example
/// ```rust
/// extern crate holochain_conductor_api;
/// extern crate holochain_core_types;
/// #[macro_use]
/// extern crate structopt;
///
/// use holochain_conductor_api::{
///     config::{load_configuration, Configuration},
///     conductor::Conductor,
/// };
/// use holochain_core_types::error::HolochainError;
/// use std::{fs::File, io::prelude::*, path::PathBuf};
/// use structopt::StructOpt;
///
/// #[derive(StructOpt, Debug)]
/// #[structopt(name = "hcc")]
/// struct Opt {
///     /// Path to the toml configuration file for the conductor
///     #[structopt(short = "c", long = "config", parse(from_os_str))]
///     config: Option<PathBuf>,
/// }
///
/// fn main() {
///     let opt = Opt::from_args();
///     let config_path = opt.config
///         .unwrap_or(PathBuf::from(r"~/.holochain/conductor/conductor_config.toml"));
///     let config_path_str = config_path.to_str().unwrap();
///     println!("Using config path: {}", config_path_str);
///     match bootstrap_from_config(config_path_str) {
///         Ok(mut conductor) => {
///             if conductor.instances().len() > 0 {
///                 println!(
///                     "Successfully loaded {} instance configurations",
///                     conductor.instances().len()
///                 );
///                 println!("Starting all of them...");
///                 conductor.start_all_instances();
///                 println!("Done.");
///                 loop {}
///             } else {
///                 println!("No instance started, bailing...");
///             }
///         }
///         Err(error) => println!("Error while trying to boot from config: {:?}", error),
///     };
/// }
///
/// fn bootstrap_from_config(path: &str) -> Result<Conductor, HolochainError> {
///     let config = load_config_file(&String::from(path))?;
///     config
///         .check_consistency()
///         .map_err(|string| HolochainError::ConfigError(string))?;
///     let mut conductor = Conductor::from_config(config);
///     conductor.boot_from_config(None)?;
///     Ok(conductor)
/// }
///
/// fn load_config_file(path: &String) -> Result<Configuration, HolochainError> {
///     let mut f = File::open(path)?;
///     let mut contents = String::new();
///     f.read_to_string(&mut contents)?;
///     Ok(load_configuration::<Configuration>(&contents)?)
/// }
/// ```
extern crate holochain_cas_implementations;
extern crate holochain_common;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_dpki;
extern crate holochain_net;
extern crate holochain_sodium;

extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;
extern crate colored;
#[cfg(test)]
extern crate holochain_wasm_utils;
extern crate jsonrpc_http_server;
extern crate jsonrpc_ws_server;
extern crate petgraph;
extern crate regex;
#[macro_use]
extern crate serde_json;
#[cfg(test)]
extern crate test_utils;
extern crate toml;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate lazy_static;
extern crate directories;
extern crate hyper;
extern crate hyper_staticfile;
extern crate json_patch;
// #[cfg(test)]
// extern crate reqwest;
extern crate tokio;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate base64;
extern crate crossbeam_channel;

pub mod conductor;
pub mod config;
pub mod context_builder;
pub mod dpki_instance;
pub mod error;
pub mod holo_signing_service;
pub mod holochain;
pub mod interface;
pub mod interface_impls;
pub mod key_loaders;
pub mod keystore;
pub mod logger;
pub mod static_file_server;

pub use crate::holochain::Holochain;
