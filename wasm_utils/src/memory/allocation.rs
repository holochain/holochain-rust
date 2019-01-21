use holochain_core_types::{error::HolochainError, json::JsonString};
use memory::{MemoryBits, MemoryInt, MEMORY_INT_MAX};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Offset(MemoryInt);
#[derive(Copy, Clone, Debug, PartialEq)]
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
    offset: Offset,
    length: Length,
}

impl WasmAllocation {
    // represent the max as MemoryBits type to allow gt comparisons
    pub fn max() -> MemoryBits {
        MEMORY_INT_MAX
    }

    pub fn new(offset: Offset, length: Length) -> AllocationResult {
        if (MemoryBits::from(offset) + MemoryBits::from(length)) > WasmAllocation::max() {
            Err(AllocationError::OutOfBounds)
        } else if MemoryInt::from(length) == 0 {
            Err(AllocationError::ZeroLength)
        } else {
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

pub type AllocationResult = Result<WasmAllocation, AllocationError>;

#[cfg(test)]
pub mod tests {

    use memory::{
        allocation::{Length, Offset, AllocationError},
        MemoryBits, MemoryInt,
    };
    use holochain_core_types::error::HolochainError;
    use memory::MEMORY_INT_MAX;
    use memory::allocation::WasmAllocation;

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
        assert_eq!(
            MEMORY_INT_MAX,
            WasmAllocation::max(),
        );
    }

    #[test]
    pub fn allocation_new_test() {
        assert_eq!(
            Err(AllocationError::OutOfBounds),
            WasmAllocation::new(Offset::from(std::u16::MAX), Length::from(1)),
        );

        assert_eq!(
            Err(AllocationError::ZeroLength),
            WasmAllocation::new(Offset::from(1), Length::from(0)),
        );

        assert_eq!(
            Ok(WasmAllocation { offset: Offset::from(1), length: Length::from(1) }),
            WasmAllocation::new(Offset::from(1), Length::from(1)),
        );
    }

    #[test]
    pub fn allocation_offset_test() {
        assert_eq!(
            Offset::from(1),
            WasmAllocation::new(Offset::from(1), Length::from(1)).unwrap().offset(),
        );
    }

    #[test]
    pub fn allocation_length_test() {
        assert_eq!(
            Length::from(1),
            WasmAllocation::new(Offset::from(1), Length::from(1)).unwrap().length(),
        );
    }

}
