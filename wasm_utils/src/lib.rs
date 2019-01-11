//! Library holding necessary code for the Ribosome  that is also useful for hdk-rust,
//! or more generally for making rust code that the Ribosome can run.
//! Must not have any dependency with any other Holochain crates.
#![feature(try_from)]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
pub extern crate holochain_core_types;
#[macro_use]
pub extern crate holochain_core_types_derive;

/// ignore api_serialization because it is nothing but structs to hold serialization
#[cfg_attr(tarpaulin, skip)]
pub mod api_serialization;

pub mod macros;
pub mod memory;

pub fn wasm_target_dir(test_path: &str, wasm_path: &str) -> String {
    match std::env::var("HC_TARGET_PREFIX") {
        Ok(prefix) => format!("{}{}{}target", prefix, test_path, wasm_path),
        Err(_) => format!("{}target", wasm_path),
    }
}
