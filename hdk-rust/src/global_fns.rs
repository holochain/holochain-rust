use globals::G_MEM_STACK;
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};
use serde;

/// Init global memory stack
pub fn init_global_memory(encoded_allocation_of_input: u32) {
    unsafe {
        G_MEM_STACK =
            Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }
}

/// Serialize output as json in WASM memory
pub fn store_and_return_output<T: serde::Serialize>(output: T) -> u32 {
    unsafe { return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), output) as u32 }
}
