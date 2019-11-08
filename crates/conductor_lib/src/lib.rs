//! let file_system = Arc::new(RwLock::new(FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap()));
//!     Arc::new(Mutex::new(SimplePersister::new(file_system.clone()))),
//!     file_system.clone(),

#![feature(try_trait, async_await)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate holochain_core;

#[macro_use]
extern crate holochain_json_derive;










#[macro_use]
extern crate log;





#[macro_use]
extern crate serde_derive;








#[macro_use]
extern crate serde_json;


#[macro_use]
extern crate maplit;
#[macro_use]
extern crate lazy_static;


#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;


#[macro_use]
extern crate nickel;

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
pub mod signal_wrapper;
pub mod static_file_server;
pub mod static_server_impls;

pub use crate::holochain::Holochain;
