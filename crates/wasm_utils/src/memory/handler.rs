use crate::memory::allocation::AllocationResult;
use holochain_json_api::json::JsonString;
use memory::{
    allocation::{AllocationError, Length, Offset, WasmAllocation},
    MemoryBits, MemoryInt, MEMORY_INT_MAX, RESERVED,
};
use std::convert::TryInto;
use crate::memory::Top;

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
    fn read_bytes(&self, offset: Offset, length: Length) -> &[u8];

    fn read_string(&self, offset: Offset, length: Length) -> String;
    /// Write a string in wasm memory according to stack state.
    fn write_string(&mut self, s: &str) -> AllocationResult {
        self.write_bytes(s.as_bytes())
    }

    fn get_top(&self) -> Top {
        let (top_bytes, _) = self.read_bytes(0, RESERVED).split_at(RESERVED as usize);
        Top::from_le_bytes(top_bytes.try_into().unwrap())
    }
    fn set_top(&self, Top) -> Top;

    // let length = max(bytes.len(), 1) as MemoryInt; // always allocate at least 1 byte
    // if MemoryBits::from(length) > WasmStack::max() {
    //     return Err(AllocationError::OutOfBounds);
    // }
    // let next_allocation = self.next_allocation(length)?;
    // let ptr = MemoryInt::from(self.allocate(next_allocation)?) as *mut c_char;
    // let ptr_safe = unsafe { slice::from_raw_parts_mut(ptr, usize::from(length)) };
    // for (i, byte) in bytes.iter().enumerate() {
    //     ptr_safe[i] = *byte as c_char;
    // }
    //
    // WasmAllocation::new((ptr as MemoryInt).into(), length)

    fn next_allocation(&self, length: Length) -> Result<WasmAllocation, AllocationError> {
        WasmAllocation::new(MemoryInt::from(self.get_top()).into(), length)
    }

    fn allocate(&mut self, allocation: WasmAllocation) -> Result<Top, AllocationError> {
        let top = self.get_top();
        if MemoryInt::from(top) != MemoryInt::from(allocation.offset()) {
            Err(AllocationError::BadStackAlignment)
        } else if MemoryBits::from(top) + MemoryBits::from(allocation.length()) > Self::max() {
            Err(AllocationError::OutOfBounds)
        } else {
            let old_top = top;
            self.set_top(allocation.offset() + allocation.length());
            Ok(old_top)
        }
    }

    fn deallocate(&mut self, allocation: WasmAllocation) -> Result<Top, AllocationError> {
        let top = self.get_top();
        if MemoryInt::from(top)
            != MemoryInt::from(allocation.offset()) + MemoryInt::from(allocation.length())
        {
            Err(AllocationError::BadStackAlignment)
        } else if MemoryInt::from(allocation.offset()) < Self::min() {
            Err(AllocationError::OutOfBounds)
        } else {
            let old_top = top;
            self.set_top(allocation.offset());
            Ok(old_top)
        }
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
