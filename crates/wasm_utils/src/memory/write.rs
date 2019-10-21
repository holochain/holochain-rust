use crate::memory::{
    allocation::{AllocationError, WasmAllocation},
    stack::WasmStack,
    MemoryBits, MemoryInt,
};
use holochain_json_api::json::JsonString;
use memory::allocation::{AllocationResult, Length};
use std::{cmp::max, convert::TryInto, os::raw::c_char, slice};

impl WasmStack {
    /// Write in wasm memory according to stack state.
    fn write_in_wasm_memory(&mut self, bytes: &[u8], length: Length) -> AllocationResult {
        let next_allocation = self.next_allocation(length)?;

        let ptr = MemoryInt::from(self.allocate(next_allocation)?) as *mut c_char;
        let ptr_safe = unsafe { slice::from_raw_parts_mut(ptr, usize::from(length)) };
        for (i, byte) in bytes.iter().enumerate() {
            ptr_safe[i] = *byte as c_char;
        }

        WasmAllocation::new((ptr as MemoryInt).into(), length)
    }

    /// Write a string in wasm memory according to stack state.
    pub fn write_string(&mut self, s: &str) -> AllocationResult {
        let bytes = s.as_bytes();
        let length = max(bytes.len(), 1) as MemoryInt; // always allocate at least 1 byte
        if MemoryBits::from(length) > WasmStack::max() {
            return Err(AllocationError::OutOfBounds);
        }

        self.write_in_wasm_memory(bytes, Length::from(length))
    }

    /// Write a data struct as a json string in wasm memory according to stack state.
    pub fn write_json<J: TryInto<JsonString>>(&mut self, jsonable: J) -> AllocationResult {
        let j: JsonString = jsonable
            .try_into()
            .map_err(|_| AllocationError::Serialization)?;

        let json_bytes = j.to_bytes();
        let json_bytes_len = max(json_bytes.len(), 1) as MemoryInt; // always allocate at least 1 byte
        if MemoryBits::from(json_bytes_len) > WasmStack::max() {
            return Err(AllocationError::OutOfBounds);
        }
        self.write_in_wasm_memory(&json_bytes, Length::from(json_bytes_len))
    }
}
