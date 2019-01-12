use std::slice;
use crate::memory::allocation::WasmAllocation;
use crate::memory::stack::WasmStack;
use crate::memory::allocation::AllocationError;
use std::os::raw::c_char;
use crate::memory::MemoryInt;
use std::convert::TryInto;
use holochain_core_types::json::JsonString;
use crate::memory::MemoryBits;
use memory::allocation::Length;

impl WasmStack {

    /// Write in wasm memory according to stack state.
    fn write_in_wasm_memory(
        &mut self,
        bytes: &[u8],
        length: Length,
    ) -> Result<WasmAllocation, AllocationError> {
        let next_allocation = self.next_allocation(length)?;

        let ptr = MemoryInt::from(self.allocate(next_allocation)?) as *mut c_char;
        let ptr_safe = unsafe { slice::from_raw_parts_mut(ptr, usize::from(length)) };
        for (i, byte) in bytes.iter().enumerate() {
            ptr_safe[i] = *byte as i8;
        }

        WasmAllocation::new((ptr as MemoryInt).into(), length)

    }

    /// Write a string in wasm memory according to stack state.
    pub fn write_string(
        &mut self,
        s: &str,
    ) -> Result<WasmAllocation, AllocationError> {
        let bytes = s.as_bytes();
        let length = bytes.len() as MemoryInt;
        if MemoryBits::from(length) > WasmStack::max() {
            return Err(AllocationError::OutOfBounds);
        }

        self.write_in_wasm_memory(bytes, Length::from(length))
    }

    /// Write a data struct as a json string in wasm memory according to stack state.
    pub fn write_json<J: TryInto<JsonString>>(
        &mut self,
        jsonable: J,
    ) -> Result<WasmAllocation, AllocationError> {
        let j: JsonString = jsonable
            .try_into()
            .map_err(|_| AllocationError::Serialization)?;

        let json_bytes = j.into_bytes();
        let json_bytes_len = json_bytes.len() as MemoryInt;
        if MemoryBits::from(json_bytes_len) > WasmStack::max() {
            return Err(AllocationError::OutOfBounds);
        }
        self.write_in_wasm_memory(&json_bytes, Length::from(json_bytes_len))
    }

}
