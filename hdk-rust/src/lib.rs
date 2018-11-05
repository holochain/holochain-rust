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
#![feature(try_from)]
#![feature(never_type)]
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

pub mod error;
pub mod api;
pub mod entry_definition;
pub mod global_fns;
pub mod globals;
pub mod init_globals;
pub mod macros;
use serde::{Serialize, Serializer};
use std::convert::TryInto;

use self::RibosomeError::*;
use globals::*;
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{
    holochain_core_types::json::JsonString, memory_allocation::*, memory_serialization::*,
};

pub mod meta;

pub use api::*;
pub use holochain_core_types::validation::*;

pub fn init_memory_stack(encoded_allocation_of_input: u32) {
    // Actual program
    // Init memory stack
    unsafe {
        G_MEM_STACK =
            Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }
}

pub fn serialize_wasm_output<J: TryInto<JsonString>>(jsonable: J) -> u32 {
    // Serialize output in WASM memory
    unsafe { store_as_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), jsonable) as u32 }
}

//--------------------------------------------------------------------------------------------------
// SYSTEM CONSTS
//--------------------------------------------------------------------------------------------------
/*
// HC.Version
const VERSION: u16 = 1;
const VERSION_STR: &'static str = "1";
*/
// HC.HashNotFound
#[derive(Debug)]
pub enum RibosomeError {
    RibosomeFailed(String),
    FunctionNotImplemented,
    HashNotFound,
    ValidationFailed(String),
}

impl Serialize for RibosomeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&match self {
            RibosomeFailed(ref error_desc) => error_desc.to_owned(),
            FunctionNotImplemented => String::from("Function not implemented"),
            HashNotFound => String::from("Hash not found"),
            ValidationFailed(ref msg) => format!("Validation failed: {}", msg),
        })
    }
}

impl From<RibosomeError> for JsonString {
    fn from(ribosome_error: RibosomeError) -> JsonString {
        JsonString::from(
            serde_json::to_string(&ribosome_error).expect("could not Jsonify RibosomeError"),
        )
    }
}
