//! The virtual machine that runs DNA written in WASM
extern crate holochain_logging;
extern crate holochain_core_types;
extern crate holochain_json_api;

// pub mod callback;
pub mod factories;
#[autotrace]
mod run_dna;
pub mod runtime;
pub use self::{run_dna::*, runtime::*};
use std::str::FromStr;
pub mod callback;
use std::path::PathBuf;
pub mod api;

pub const MAX_ZOME_CALLS: usize = 10;

pub trait Defn: FromStr {
    /// return the canonical name of this function definition
    fn as_str(&self) -> &'static str;

    /// convert the canonical name of this function to an index
    fn str_to_index(s: &str) -> usize;

    /// convert an index to the function definition
    fn from_index(i: usize) -> Self;
}

pub fn wasm_target_dir(test_path: &PathBuf, wasm_path: &PathBuf) -> PathBuf {
    // this env var checker can't use holochain_common
    // crate because that uses `directories` crate which doesn't compile to WASM
    let mut target_dir = PathBuf::new();
    if let Ok(prefix) = std::env::var("HC_TARGET_PREFIX") {
        target_dir.push(PathBuf::from(prefix));
        target_dir.push("crates");
        target_dir.push(test_path);
    }
    target_dir.push(wasm_path);
    target_dir.push(PathBuf::from("target"));

    target_dir
}
