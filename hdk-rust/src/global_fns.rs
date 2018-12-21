//! This file contains small helper functions relating to WASM memory management
//! and serialization used throughout the HDK.

use crate::globals::G_MEM_STACK;
use holochain_core_types::json::JsonString;
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};
use std::convert::TryInto;

/// Init global memory stack
pub fn init_global_memory(encoded_allocation_of_input: u32) {
    unsafe {
        G_MEM_STACK =
            Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }
}

/// Serialize output as json in WASM memory
pub fn store_and_return_output<J: TryInto<JsonString>>(jsonable: J) -> u32 {
    unsafe {
        return store_as_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), jsonable) as u32;
    }
}
