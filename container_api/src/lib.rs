//! let file_system = Arc::new(RwLock::new(FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap()));
//!     Arc::new(Mutex::new(SimplePersister::new(file_system.clone()))),
//!     file_system.clone(),

#![feature(try_from, async_await, await_macro)]

/// Holochain Container API
///
/// This crate is a library that provides types and functions that help with building
/// a Holochain Container as described in [ADR15](doc/architecture/decisions/0015-container-api).
///
/// Depending on the specific (application) use-case, the context in which a Holochain instance
/// is run may vary drastically. Application developers may want to bundle Holochain with
/// and statically link the core library into their custom made executable.
/// In such a case, [holochain.rs](container_api/src/holochain.rs) may be used directly as a
/// wrapper around a single instance.
///
/// In the general case, many different DNAs are being executed alongside each other in the
/// context of the same agent, i.e. user. [container.rs](container_api/src/container.rs) provides
/// a struct that instantiates, holds, manages several Holochain instances.
/// It makes use of [config.rs](container_api/src/config.rs) which provides structs for container
/// configuration that can be de-/serialized into config files like
/// [these](https://hackmd.io/OcT2cI1ETfu4DHyvn4QZ5A#).
///
/// All of the above is used in the [container crate](container).
///
/// # Example
/// ```rust
/// #![feature(try_from)]
/// extern crate clap;
/// extern crate holochain_container_api;
/// extern crate holochain_core_types;
/// #[macro_use]
/// extern crate structopt;
///
/// use holochain_container_api::{
///     config::{load_configuration, Configuration},
///     container::Container,
/// };
/// use holochain_core_types::error::HolochainError;
/// use std::{convert::TryFrom, fs::File, io::prelude::*, path::PathBuf};
/// use structopt::StructOpt;
///
/// #[derive(StructOpt, Debug)]
/// #[structopt(name = "hcc")]
/// struct Opt {
///     /// Output file
///     #[structopt(short = "c", long = "config", parse(from_os_str))]
///     config: Option<PathBuf>,
/// }
///
/// fn main() {
///     let opt = Opt::from_args();
///     let config_path = opt.config
///         .unwrap_or(PathBuf::from(r"~/.holochain/container_config.toml"));
///     let config_path_str = config_path.to_str().unwrap();
///     println!("Using config path: {}", config_path_str);
///     match bootstrap_from_config(config_path_str) {
///         Ok(mut container) => {
///             if container.instances().len() > 0 {
///                 println!(
///                     "Successfully loaded {} instance configurations",
///                     container.instances().len()
///                 );
///                 println!("Starting all of them...");
///                 container.start_all_instances();
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
/// fn bootstrap_from_config(path: &str) -> Result<Container, HolochainError> {
///     let config = load_config_file(&String::from(path))?;
///     config
///         .check_consistency()
///         .map_err(|string| HolochainError::ConfigError(string))?;
///     Container::try_from(&config)
/// }
///
/// fn load_config_file(path: &String) -> Result<Configuration, HolochainError> {
///     let mut f = File::open(path)?;
///     let mut contents = String::new();
///     f.read_to_string(&mut contents)?;
///     Ok(load_configuration::<Configuration>(&contents)?)
/// }
/// ```
extern crate futures;
extern crate holochain_cas_implementations;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_net;

extern crate serde;
extern crate tempfile;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;
#[cfg(test)]
extern crate holochain_wasm_utils;
extern crate jsonrpc_ws_server;
extern crate jsonrpc_http_server;
extern crate petgraph;
extern crate serde_json;
#[cfg(test)]
extern crate test_utils;
extern crate tiny_http;
extern crate toml;

pub mod config;
pub mod container;
pub mod context_builder;
pub mod error;
pub mod holochain;
pub mod interface;
pub mod interface_impls;

pub use crate::holochain::Holochain;
