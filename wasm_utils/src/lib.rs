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
pub mod memory_allocation;
pub mod memory_serialization;

pub fn wasm_target_dir(fallback: &str) -> String {
    match std::env::var("HC_TARGET_PREFIX") {
        Ok(target_prefix) => {
            let test_path = std::env::var("TEST_PATH").unwrap_or(String::new());
            let wasm_path = std::env::var("WASM_PATH").unwrap_or(String::new());
            format!("{}{}{}target", target_prefix, test_path, wasm_path)
        },
        Err(_) => fallback.to_string(),
    }
}
