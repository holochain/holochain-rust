// extends memory allocation to work with ribosome encodings

use holochain_core_types::{
    bits_n_pieces::{u32_merge_bits, u32_split_bits},
    error::{
        HolochainError, RibosomeEncodedAllocation, RibosomeEncodedValue, RibosomeEncodingBits,
        RibosomeErrorCode,
    },
    json::JsonString,
};
use memory::{
    allocation::{AllocationError, AllocationResult, WasmAllocation},
    stack::WasmStack,
    MemoryBits,
};
use std::convert::TryFrom;

impl TryFrom<RibosomeEncodedAllocation> for WasmAllocation {
    type Error = AllocationError;
    fn try_from(
        ribosome_memory_allocation: RibosomeEncodedAllocation,
    ) -> Result<Self, Self::Error> {
        let (offset, length) = u32_split_bits(MemoryBits::from(ribosome_memory_allocation));
        WasmAllocation::new(offset.into(), length.into())
    }
}

impl From<WasmAllocation> for RibosomeEncodedAllocation {
    fn from(wasm_allocation: WasmAllocation) -> Self {
        u32_merge_bits(
            wasm_allocation.offset().into(),
            wasm_allocation.length().into(),
        )
        .into()
    }
}

impl From<WasmAllocation> for RibosomeEncodedValue {
    fn from(wasm_allocation: WasmAllocation) -> Self {
        RibosomeEncodedValue::Allocation(RibosomeEncodedAllocation::from(wasm_allocation))
    }
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

impl From<AllocationError> for RibosomeEncodedValue {
    fn from(allocation_error: AllocationError) -> Self {
        RibosomeEncodedValue::Failure(RibosomeErrorCode::from(allocation_error))
    }
}

impl AllocationError {
    pub fn as_ribosome_encoding(&self) -> RibosomeEncodingBits {
        RibosomeEncodedValue::from(self.clone()).into()
    }
}

impl WasmAllocation {
    /// equivalent to TryFrom<RibosomeEncodingBits> for WasmAllocation
    /// not implemented as a trait because RibosomeEncodingBits is a primitive and that would couple
    /// allocations to ribosome encoding
    pub fn try_from_ribosome_encoding(encoded_value: RibosomeEncodingBits) -> AllocationResult {
        match RibosomeEncodedValue::from(encoded_value) {
            RibosomeEncodedValue::Success => Err(AllocationError::ZeroLength),
            RibosomeEncodedValue::Failure(_) => Err(AllocationError::OutOfBounds),
            RibosomeEncodedValue::Allocation(ribosome_allocation) => {
                WasmAllocation::try_from(ribosome_allocation)
            }
        }
    }

    pub fn as_ribosome_encoding(&self) -> RibosomeEncodingBits {
        RibosomeEncodedValue::from(self.clone()).into()
    }
}

impl WasmStack {
    /// equivalent to TryFrom<RibosomeEncodingBits> for WasmStack
    /// not implemented as a trait because RibosomeEncodingBits is a primitive and that would couple
    /// stacks to ribosome encoding
    /// wraps WasmAllocation::try_from_ribosome_encoding internally but has a "higher level"
    /// return signature intended for direct use/return in/from ribosome fns
    pub fn try_from_ribosome_encoding(
        maybe_encoded_allocation: RibosomeEncodingBits,
    ) -> Result<WasmStack, RibosomeEncodedValue> {
        match WasmAllocation::try_from_ribosome_encoding(maybe_encoded_allocation) {
            Err(allocation_error) => Err(allocation_error),
            Ok(allocation) => match WasmStack::try_from(allocation) {
                Err(allocation_error) => Err(allocation_error),
                Ok(stack) => Ok(stack),
            },
        }
        .map_err(|e| e.as_ribosome_encoding().into())
    }
}

/// Equivalent to From<AllocationResult> for RibosomeEncodedValue
/// not possible to implement the trait as Result and RibosomeEncodedValue from different crates
pub fn return_code_for_allocation_result(result: AllocationResult) -> RibosomeEncodedValue {
    match result {
        Ok(allocation) => RibosomeEncodedValue::from(allocation),
        Err(allocation_error) => RibosomeEncodedValue::from(allocation_error),
    }
}

pub fn load_ribosome_encoded_string(
    encoded_value: RibosomeEncodingBits,
) -> Result<String, HolochainError> {
    // almost the same as WasmAllocation::try_from_ribosome_encoding but maps to HolochainError
    match RibosomeEncodedValue::from(encoded_value) {
        RibosomeEncodedValue::Success => Err(HolochainError::Ribosome(
            RibosomeErrorCode::ZeroSizedAllocation,
        ))?,
        RibosomeEncodedValue::Failure(err_code) => Err(HolochainError::Ribosome(err_code))?,
        RibosomeEncodedValue::Allocation(ribosome_allocation) => {
            Ok(WasmAllocation::try_from(ribosome_allocation)?.read_to_string())
        }
    }
}

pub fn load_ribosome_encoded_json<J: TryFrom<JsonString>>(
    encoded_value: RibosomeEncodingBits,
) -> Result<J, HolochainError>
where
    J::Error: Into<HolochainError>,
{
    let s = load_ribosome_encoded_string(encoded_value)?;
    let j = JsonString::from(s);

    J::try_from(j).map_err(|e| e.into())
}

#[cfg(test)]
pub mod tests {

    use memory::allocation::AllocationError;
    use memory::allocation::WasmAllocation;
    use memory::stack::WasmStack;
    use memory::allocation::Offset;
    use memory::allocation::Length;
    use memory::stack::Top;
    use holochain_core_types::error::RibosomeEncodedAllocation;
    use holochain_core_types::error::RibosomeEncodingBits;
    use holochain_core_types::error::RibosomeEncodedValue;
    use holochain_core_types::error::RibosomeErrorCode;
    use std::convert::TryFrom;
    use holochain_core_types::bits_n_pieces::u32_merge_bits;
    use memory::ribosome::return_code_for_allocation_result;

    #[test]
    fn try_allocation_from_ribosome_allocation_test() {
        assert_eq!(
            Err(AllocationError::ZeroLength),
            WasmAllocation::try_from(RibosomeEncodedAllocation::from(0)),
        );

        assert_eq!(
            Err(AllocationError::OutOfBounds),
            WasmAllocation::try_from(RibosomeEncodedAllocation::from(u32_merge_bits(std::u16::MAX, std::u16::MAX))),
        );

        assert_eq!(
            Ok(WasmAllocation{ offset: Offset::from(4), length: Length::from(8) }),
            WasmAllocation::try_from(RibosomeEncodedAllocation::from(0b00000000000000100_0000000000001000)),
        );

    }

    #[test]
    fn ribosome_allocation_from_allocation_test() {
        assert_eq!(
            RibosomeEncodedAllocation::from(0b0000000000000100_0000000000001000),
            RibosomeEncodedAllocation::from(WasmAllocation{ offset: Offset::from(4), length: Length::from(8) }),
        );
    }

    #[test]
    fn ribosome_encoded_value_from_allocation_test() {
        assert_eq!(
            RibosomeEncodedValue::Allocation(RibosomeEncodedAllocation::from(0b0000000000000100_0000000000001000)),
            RibosomeEncodedValue::from(WasmAllocation{ offset: Offset::from(4), length: Length::from(8) }),
        );
    }

    #[test]
    fn ribosome_error_from_allocation_error_test() {
        assert_eq!(
            RibosomeErrorCode::OutOfMemory,
            RibosomeErrorCode::from(AllocationError::OutOfBounds),
        );

        assert_eq!(
            RibosomeErrorCode::ZeroSizedAllocation,
            RibosomeErrorCode::from(AllocationError::ZeroLength),
        );

        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            RibosomeErrorCode::from(AllocationError::BadStackAlignment),
        );

        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            RibosomeErrorCode::from(AllocationError::Serialization),
        );
    }

    #[test]
    fn ribosome_code_from_allocation_error_test() {
        assert_eq!(
            RibosomeEncodedValue::Failure(RibosomeErrorCode::OutOfMemory),
            RibosomeEncodedValue::from(AllocationError::OutOfBounds),
        );

        assert_eq!(
            RibosomeEncodedValue::Failure(RibosomeErrorCode::ZeroSizedAllocation),
            RibosomeEncodedValue::from(AllocationError::ZeroLength),
        );

        assert_eq!(
            RibosomeEncodedValue::Failure(RibosomeErrorCode::NotAnAllocation),
            RibosomeEncodedValue::from(AllocationError::BadStackAlignment),
        );

        assert_eq!(
            RibosomeEncodedValue::Failure(RibosomeErrorCode::NotAnAllocation),
            RibosomeEncodedValue::from(AllocationError::Serialization),
        );
    }

    #[test]
    fn ribosome_encoding_test() {
        assert_eq!(
            RibosomeEncodingBits::from(RibosomeEncodedValue::Failure(RibosomeErrorCode::OutOfMemory)),
            AllocationError::OutOfBounds.as_ribosome_encoding(),
        );
        assert_eq!(
            RibosomeEncodingBits::from(RibosomeEncodedValue::Failure(RibosomeErrorCode::ZeroSizedAllocation)),
            AllocationError::ZeroLength.as_ribosome_encoding(),
        );
        assert_eq!(
            RibosomeEncodingBits::from(RibosomeEncodedValue::Failure(RibosomeErrorCode::NotAnAllocation)),
            AllocationError::BadStackAlignment.as_ribosome_encoding(),
        );
        assert_eq!(
            RibosomeEncodingBits::from(RibosomeEncodedValue::Failure(RibosomeErrorCode::NotAnAllocation)),
            AllocationError::Serialization.as_ribosome_encoding(),
        );
    }

    #[test]
    fn stack_from_encoding_test() {
        assert_eq!(
            Err(RibosomeEncodedValue::from(AllocationError::OutOfBounds)),
            WasmStack::try_from_ribosome_encoding(u32_merge_bits(std::u16::MAX, std::u16::MAX)),
        );

        assert_eq!(
            Err(RibosomeEncodedValue::from(AllocationError::ZeroLength)),
            WasmStack::try_from_ribosome_encoding(0),
        );

        assert_eq!(
            Ok(WasmStack{ top: Top(4) }),
            // 2 + 2 = 4
            WasmStack::try_from_ribosome_encoding(0b0000000000000010_0000000000000010),
        );
    }

    #[test]
    fn return_code_for_allocation_result_test() {
        assert_eq!(
            RibosomeEncodedValue::from(AllocationError::OutOfBounds),
            return_code_for_allocation_result(Err(AllocationError::OutOfBounds)),
        );
        assert_eq!(
            RibosomeEncodedValue::from(AllocationError::ZeroLength),
            return_code_for_allocation_result(Err(AllocationError::ZeroLength)),
        );
        assert_eq!(
            RibosomeEncodedValue::from(AllocationError::BadStackAlignment),
            return_code_for_allocation_result(Err(AllocationError::BadStackAlignment)),
        );
        assert_eq!(
            RibosomeEncodedValue::from(AllocationError::Serialization),
            return_code_for_allocation_result(Err(AllocationError::Serialization)),
        );
        let allocation = WasmAllocation{ offset: Offset::from(5), length: Length::from(5) };
        assert_eq!(
            RibosomeEncodedValue::from(allocation),
            return_code_for_allocation_result(Ok(allocation)),
        );
    }

}
