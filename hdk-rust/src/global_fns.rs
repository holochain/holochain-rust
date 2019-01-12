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
pub fn write_json<J: TryInto<JsonString>>(jsonable: J) -> WasmAllocation {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };
    mem_stack.write_json(jsonable)
}
