//! The virtual machine that runs DNA written in WASM

pub mod api;
pub mod callback;
pub mod fn_call;
pub mod memory;
mod run_dna;
mod runtime;

pub use self::{run_dna::*, runtime::*};

use holochain_core_types::dna::capabilities::ReservedCapabilityNames;

use std::str::FromStr;

pub trait Defn: FromStr {
    /// return the canonical name of this function definition
    fn as_str(&self) -> &'static str;

    /// convert the canonical name of this function to an index
    fn str_to_index(s: &str) -> usize;

    /// convert an index to the function definition
    fn from_index(i: usize) -> Self;
}
