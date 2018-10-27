//! Holochain Development Kit (HDK)
//!
//! The HDK helps in writing Holochain applications.
//! Holochain DNAs need to be written in WebAssembly, or a language that compiles to Wasm,
//! such as Rust. The HDK handles some of the low-level details of Wasm execution like
//! memory allocation, (de)serializing data, and shuffling data and functions into and out of Wasm
//! memory via some helper functions and Holochain-specific macros.
//!
//! The HDK lets the developer focus on application logic and, as much as possible, forget about the
//! underlying low-level implementation. It would be possible to write DNA source code without an
//! HDK, but it would be extremely tedious!

pub extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
pub extern crate holochain_core_types;
pub extern crate holochain_dna;
pub extern crate holochain_wasm_utils;

mod api;
pub mod entry_definition;
pub mod error;
pub mod global_fns;
pub mod globals;
pub mod init_globals;
pub mod macros;
pub mod meta;

pub use api::*;
pub use holochain_core_types::validation::*;
