use crate::memory::allocation::AllocationResult;
use holochain_json_api::json::JsonString;
use memory::{
    allocation::{AllocationError, WasmAllocation},
    MemoryBits, MemoryInt, MEMORY_INT_MAX, RESERVED,
};
use std::convert::TryInto;

pub trait WasmMemoryHandler {
    // represent the max as MemoryBits type to allow gt comparisons
    fn max() -> MemoryBits {
        MEMORY_INT_MAX
    }

    // min compares lt so can be a MemoryInt
    fn min() -> MemoryInt {
        RESERVED
    }

    /// Write in wasm memory according to stack state.
    fn write_bytes(&mut self, bytes: &[u8]) -> AllocationResult;
    fn read_bytes(&self, allocation: WasmAllocation) -> &[u8];

    fn read_string(&self, allocation: WasmAllocation) -> String;
    /// Write a string in wasm memory according to stack state.
    fn write_string(&mut self, s: &str) -> AllocationResult {
        self.write_bytes(s.as_bytes())
    }

    /// Write a data struct as a json string in wasm memory according to stack state.
    fn write_json<J: TryInto<JsonString>>(&mut self, jsonable: J) -> AllocationResult {
        let j: JsonString = jsonable
            .try_into()
            .map_err(|_| AllocationError::Serialization)?;

        let json_bytes = j.to_bytes();
        // let json_bytes_len = max(json_bytes.len(), 1) as MemoryInt; // always allocate at least 1 byte
        // if MemoryBits::from(json_bytes_len) > Self::max() {
        //     return Err(AllocationError::OutOfBounds);
        // }
        self.write_bytes(&json_bytes)
    }
}
