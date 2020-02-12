//! The virtual machine that runs DNA written in WASM

pub mod api;
pub mod callback;
pub mod factories;
pub mod memory;
#[autotrace]
mod run_dna;
pub mod runtime;
pub use self::{run_dna::*, runtime::*};
use std::str::FromStr;

pub const MAX_ZOME_CALLS: usize = 10;

pub trait Defn: FromStr {
    /// return the canonical name of this function definition
    fn as_str(&self) -> &'static str;

    /// convert the canonical name of this function to an index
    fn str_to_index(s: &str) -> usize;

    /// convert an index to the function definition
    fn from_index(i: usize) -> Self;
}
