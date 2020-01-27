use holochain_core_types::error::HolochainError;
use holochain_json_api::{error::JsonError, json::JsonString};
use memory::RESERVED;
use std::convert::TryFrom;

use memory::{MemoryBits, MemoryInt, MEMORY_INT_MAX};

pub type Offset = MemoryInt;
pub type Length = MemoryInt;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone, PartialEq)]
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

impl From<AllocationError> for String {
    fn from(allocation_error: AllocationError) -> Self {
        match allocation_error {
            AllocationError::OutOfBounds => "Allocation out of bounds",
            AllocationError::ZeroLength => "Allocation is zero length",
            AllocationError::BadStackAlignment => "Allocation not aligned with stack",
            AllocationError::Serialization => "Allocation could not serialize data",
        }
        .into()
    }
}

impl From<AllocationError> for HolochainError {
    fn from(allocation_error: AllocationError) -> Self {
        HolochainError::ErrorGeneric(String::from(allocation_error))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WasmAllocation {
    // public fields to the crate for tests
    /// raw offset is what is passed in but doesn't include reserved bytes at the start of memory
    /// the real offset will be calculated by offset() method
    pub(in crate::memory) raw_offset: Offset,
    pub(in crate::memory) length: Length,
}

impl WasmAllocation {
    // represent the max as MemoryBits type to allow gt comparisons
    pub fn max() -> MemoryBits {
        MEMORY_INT_MAX
    }

    pub fn new(raw_offset: Offset, length: Length) -> AllocationResult {
        if (MemoryBits::from(RESERVED) + MemoryBits::from(raw_offset) + MemoryBits::from(length))
            > WasmAllocation::max()
        {
            Err(AllocationError::OutOfBounds)
        } else if MemoryInt::from(length) == 0 {
            Err(AllocationError::ZeroLength)
        } else {
            Ok(WasmAllocation { raw_offset, length })
        }
    }

    ///
    pub fn offset(self) -> Offset {
        (RESERVED as MemoryInt + MemoryInt::from(self.raw_offset)).into()
    }

    /// length in bytes
    pub fn length(self) -> Length {
        self.length
    }

    /// alias for offset
    pub fn start(self) -> Offset {
        self.offset()
    }

    /// start plus the length
    pub fn end(self) -> Offset {
        (MemoryInt::from(self.start()) + MemoryInt::from(self.length())).into()
    }
}

impl TryFrom<&str> for WasmAllocation {
    type Error = AllocationError;
    fn try_from(s: &str) -> Result<WasmAllocation, AllocationError> {
        Ok(WasmAllocation::new(
            s.as_ptr() as Offset,
            s.len() as Length,
        )?)
    }
}

impl TryFrom<String> for WasmAllocation {
    type Error = AllocationError;
    fn try_from(s: String) -> Result<WasmAllocation, AllocationError> {
        Ok(WasmAllocation::new(
            s.as_ptr() as Offset,
            s.len() as Length,
        )?)
    }
}

pub type AllocationResult = Result<WasmAllocation, AllocationError>;

#[cfg(test)]
pub mod tests {

    use holochain_core_types::{bits_n_pieces::U16_MAX, error::HolochainError};
    use memory::{
        allocation::{AllocationError, Length, Offset, WasmAllocation},
        MemoryBits, MemoryInt, MEMORY_INT_MAX,
    };

    pub fn fake_offset() -> Offset {
        Offset(12345)
    }

    pub fn fake_length() -> Length {
        Length(12345)
    }

    #[test]
    pub fn memory_int_from_offset_test() {
        assert_eq!(12345 as MemoryInt, MemoryInt::from(fake_offset()),);
    }

    #[test]
    pub fn memory_bits_from_offset_test() {
        assert_eq!(12345 as MemoryBits, MemoryBits::from(fake_offset()),);
    }

    #[test]
    pub fn offset_from_memory_int_test() {
        assert_eq!(fake_offset(), Offset::from(12345 as MemoryInt),);
    }

    #[test]
    pub fn memory_int_from_length_test() {
        assert_eq!(12345 as MemoryInt, MemoryInt::from(fake_length()),);
    }

    #[test]
    pub fn memory_bits_from_length_test() {
        assert_eq!(12345 as MemoryBits, MemoryBits::from(fake_length()),);
    }

    #[test]
    pub fn length_from_memory_int_test() {
        assert_eq!(fake_length(), Length::from(12345 as MemoryInt),);
    }

    #[test]
    pub fn usize_from_length_test() {
        assert_eq!(usize::from(fake_length()), 12345 as usize,);
    }

    #[test]
    pub fn string_from_allocation_test() {
        assert_eq!(
            String::from("Allocation out of bounds"),
            String::from(AllocationError::OutOfBounds),
        );
        assert_eq!(
            String::from("Allocation is zero length"),
            String::from(AllocationError::ZeroLength),
        );
        assert_eq!(
            String::from("Allocation not aligned with stack"),
            String::from(AllocationError::BadStackAlignment),
        );
        assert_eq!(
            String::from("Allocation could not serialize data"),
            String::from(AllocationError::Serialization),
        );
    }

    #[test]
    pub fn holochain_error_from_allocation_error_test() {
        assert_eq!(
            HolochainError::from("Allocation out of bounds"),
            HolochainError::from(AllocationError::OutOfBounds),
        );
        assert_eq!(
            HolochainError::from("Allocation is zero length"),
            HolochainError::from(AllocationError::ZeroLength),
        );
        assert_eq!(
            HolochainError::from("Allocation not aligned with stack"),
            HolochainError::from(AllocationError::BadStackAlignment),
        );
        assert_eq!(
            HolochainError::from("Allocation could not serialize data"),
            HolochainError::from(AllocationError::Serialization),
        );
    }

    #[test]
    pub fn allocation_max_test() {
        assert_eq!(MEMORY_INT_MAX, WasmAllocation::max(),);
    }

    #[test]
    pub fn allocation_new_test() {
        assert_eq!(
            Err(AllocationError::OutOfBounds),
            WasmAllocation::new(Offset::from(std::u32::MAX), Length::from(1_u32)),
        );

        assert_eq!(
            Err(AllocationError::ZeroLength),
            WasmAllocation::new(Offset::from(1_u32), Length::from(0_u32)),
        );

        assert_eq!(
            Ok(WasmAllocation {
                offset: Offset::from(1_u32),
                length: Length::from(1_u32)
            }),
            WasmAllocation::new(Offset::from(1_u32), Length::from(1_u32)),
        );

        // allocation larger than 1 wasm page
        let big = U16_MAX * 2_u32;
        assert_eq!(
            Ok(WasmAllocation {
                offset: Offset::from(big),
                length: Length::from(big),
            }),
            WasmAllocation::new(Offset::from(big), Length::from(big)),
        );
    }

    #[test]
    pub fn allocation_offset_test() {
        assert_eq!(
            Offset::from(1),
            WasmAllocation::new(Offset::from(1_u32), Length::from(1_u32))
                .unwrap()
                .offset(),
        );
    }

    #[test]
    pub fn allocation_length_test() {
        assert_eq!(
            Length::from(1_u32),
            WasmAllocation::new(Offset::from(1_u32), Length::from(1_u32))
                .unwrap()
                .length(),
        );
    }
}
