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
//!
//! Throughout the development process it will be helpful to click around through this reference, but
//! the most useful places to start reading are the [define_zome! macro](macro.define_zome.html), and the list of exposed functions
//! that Holochain offers: [the API](api/index.html).
#![warn(unused_extern_crates)]
#![feature(never_type)]

pub use serde;
pub use serde_json;
pub use serde_derive;
pub use holochain_core_types;
pub use holochain_json_derive;
pub use holochain_json_api;
pub use holochain_persistence_api;
pub use holochain_wasm_utils;
pub use pretty_assertions;

pub mod api;
pub mod utils;
#[macro_use]
pub mod entry_definition;
pub mod error;
pub mod global_fns;
pub mod init_globals;
pub mod macros;

pub use holochain_wasm_utils::api_serialization::{validation::*, THIS_INSTANCE};

pub mod meta;
pub mod prelude;

pub use crate::api::*;
pub use holochain_core_types::validation::*;
