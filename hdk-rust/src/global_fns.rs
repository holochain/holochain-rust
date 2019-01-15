//! This file contains small helper functions relating to WASM memory management
//! and serialization used throughout the HDK.

use crate::globals::G_MEM_STACK;
use holochain_core_types::json::JsonString;
pub use holochain_wasm_utils::api_serialization::validation::*;
use std::convert::TryInto;
use holochain_wasm_utils::memory::allocation::AllocationResult;
use holochain_wasm_utils::memory::stack::WasmStack;
use holochain_wasm_utils::memory::allocation::WasmAllocation;
use holochain_wasm_utils::memory::allocation::AllocationError;
use std::convert::TryFrom;

/// Init global memory stack
pub fn init_global_memory(initial_allocation: WasmAllocation) -> AllocationResult {
    unsafe {
        G_MEM_STACK =
            Some(WasmStack::try_from(initial_allocation)?);
    }
    Ok(initial_allocation)
}

/// Serialize output as json in WASM memory
pub fn write_json<J: TryInto<JsonString>>(jsonable: J) -> AllocationResult {
    let mut mem_stack = unsafe { match G_MEM_STACK {
        Some(mem_stack) => mem_stack,
        None => return Err(AllocationError::BadStackAlignment),
    } };
    mem_stack.write_json(jsonable)
}
