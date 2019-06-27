//! Library holding necessary code for the Ribosome  that is also useful for hdk-rust,
//! or more generally for making rust code that the Ribosome can run.
//! Must not have any dependency with any other Holochain crates.
#![warn(unused_extern_crates)]
#[macro_use]
extern crate serde_derive;
pub extern crate holochain_core_types;
#[macro_use]
pub extern crate holochain_json_derive;
pub extern crate holochain_json_api;
pub extern crate holochain_persistence_api;

/// ignore api_serialization because it is nothing but structs to hold serialization
#[cfg_attr(tarpaulin, skip)]
pub mod api_serialization;

pub mod macros;
pub mod memory;

use std::path::PathBuf;

pub fn wasm_target_dir(test_path: &PathBuf, wasm_path: &PathBuf) -> PathBuf {
    // this env var checker can't use holochain_common
    // crate because that uses `directories` crate which doesn't compile to WASM
    let mut target_dir = PathBuf::new();
    if let Ok(prefix) = std::env::var("HC_TARGET_PREFIX") {
        target_dir.push(PathBuf::from(prefix));
        target_dir.push(test_path);
    }
    target_dir.push(wasm_path);
    target_dir.push(PathBuf::from("target"));

    target_dir
}
