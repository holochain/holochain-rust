use holochain_core_types::error::RibosomeEncodedAllocation;
use holochain_core_types::bits_n_pieces::u32_split_bits;
use memory::MemoryBits;
use memory::MEMORY_INT_MAX;
use std::convert::TryFrom;
use memory::MemoryInt;
use holochain_core_types::json::JsonString;
use holochain_core_types::error::HolochainError;
use holochain_core_types::bits_n_pieces::u32_merge_bits;
use holochain_core_types::error::RibosomeReturnCode;
use holochain_core_types::error::RibosomeErrorCode;

#[derive(Copy, Clone, Debug)]
pub struct Offset(MemoryInt);
#[derive(Copy, Clone, Debug)]
pub struct Length(MemoryInt);

impl From<Offset> for MemoryInt {
    fn from(offset: Offset) -> Self {
        offset.0
    }
}

impl From<Offset> for MemoryBits {
    fn from(offset: Offset) -> Self {
        offset.0 as MemoryBits
    }
}

impl From<MemoryInt> for Offset {
    fn from(i: MemoryInt) -> Self {
        Offset(i)
    }
}

impl From<Length> for MemoryInt {
    fn from(length: Length) -> Self {
        length.0
    }
}

impl From<Length> for MemoryBits {
    fn from(length: Length) -> Self {
        length.0 as MemoryBits
    }
}

impl From<MemoryInt> for Length {
    fn from(i: MemoryInt) -> Self {
        Length(i)
    }
}

impl From<Length> for usize {
    fn from(length: Length) -> Self {
        length.0 as usize
    }
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub enum AllocationError {
    /// (de)allocation is either too large or implies negative values
    OutOfBounds,
    /// cannot allocate zero data
    ZeroLength,
    /// (de)allocation must occur at the top of the stack
    BadStackAlignment,
    /// writes can fail to serialize data before allocation occurs e.g. json
    Serialization,
}

impl From<AllocationError> for RibosomeErrorCode {
    fn from(allocation_error: AllocationError) -> Self {
        match allocation_error {
            AllocationError::OutOfBounds => RibosomeErrorCode::OutOfMemory,
            AllocationError::ZeroLength => RibosomeErrorCode::ZeroSizedAllocation,
            AllocationError::BadStackAlignment => RibosomeErrorCode::NotAnAllocation,
            AllocationError::Serialization => RibosomeErrorCode::NotAnAllocation,
        }
    }
}

impl From<AllocationError> for String {
    fn from(allocation_error: AllocationError) -> Self {
        match allocation_error {
            AllocationError::OutOfBounds => "Allocation out of bounds".into(),
            AllocationError::ZeroLength => "Allocation is zero length".into(),
            AllocationError::BadStackAlignment => "Allocation is not aligned with stack".into(),
            AllocationError::Serialization => "Allocation could not serialize data".into(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WasmAllocation {
    offset: Offset,
    length: Length,
}

impl WasmAllocation {
    // represent the max as MemoryBits type to allow gt comparisons
    pub fn max()-> MemoryBits {
        MEMORY_INT_MAX
    }

    pub fn new(offset: Offset, length: Length) -> Result<Self, AllocationError> {
        if (MemoryBits::from(offset) + MemoryBits::from(length)) > WasmAllocation::max() {
            Err(AllocationError::OutOfBounds)
        }
        else if MemoryInt::from(length) == 0 {
            Err(AllocationError::ZeroLength)
        }
        else {
            Ok(WasmAllocation { offset, length })
        }
    }

    pub fn offset(self) -> Offset {
        self.offset
    }

    pub fn length(self) -> Length {
        self.length
    }

}

impl TryFrom<RibosomeEncodedAllocation> for WasmAllocation {
    type Error = AllocationError;
    fn try_from(ribosome_memory_allocation: RibosomeEncodedAllocation) -> Result<Self, Self::Error> {
        let (offset, length) = u32_split_bits(MemoryBits::from(ribosome_memory_allocation));
        WasmAllocation::new(offset.into(), length.into())
    }
}

impl From<WasmAllocation> for RibosomeEncodedAllocation {
    fn from(wasm_allocation: WasmAllocation) -> Self {
        u32_merge_bits(wasm_allocation.offset().into(), wasm_allocation.length().into()).into()
    }
}

impl From<WasmAllocation> for RibosomeReturnCode {
    fn from(wasm_allocation: WasmAllocation) -> Self {
        RibosomeReturnCode::Allocation(RibosomeEncodedAllocation::from(wasm_allocation))
    }
}

impl From<AllocationError> for RibosomeReturnCode {
    fn from(allocation_error: AllocationError) -> Self {
        RibosomeReturnCode::Failure(RibosomeErrorCode::from(allocation_error))
    }
}

pub type AllocationResult = Result<WasmAllocation, AllocationError>;
pub fn return_code_for_allocation_result(result: Result<WasmAllocation, AllocationError>) -> RibosomeReturnCode {
    match result {
        Ok(allocation) => RibosomeReturnCode::from(allocation),
        Err(allocation_error) => RibosomeReturnCode::from(allocation_error),
    }
}
