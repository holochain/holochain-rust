//! This file contains small helper functions relating to WASM memory management
//! and serialization used throughout the HDK.

use crate::globals::G_MEM_STACK;
use holochain_core_types::json::JsonString;
pub use holochain_wasm_utils::api_serialization::validation::*;
use std::convert::TryInto;
use holochain_wasm_utils::memory::allocation::WasmAllocation;
use holochain_wasm_utils::memory::stack::WasmStack;

/// Init global memory stack
pub fn init_global_memory(initial_allocation: WasmAllocation) {
    unsafe {
        G_MEM_STACK =
            Some(WasmStack::from(initial_allocation));
    }
}

/// Serialize output as json in WASM memory
pub fn store_and_return_output<J: TryInto<JsonString>>(jsonable: J) -> u32 {
    unsafe {
        return store_as_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), jsonable) as u32;
    }
}
