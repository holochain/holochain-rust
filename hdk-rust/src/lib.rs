//! File holding the public Zome API
//! All API Reference documentation should be done here.

pub extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
pub extern crate holochain_wasm_utils;

pub mod api;
pub mod global_fns;
pub mod globals;
pub mod init_globals;
pub mod macros;
