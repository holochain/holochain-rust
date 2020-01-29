// extends memory allocation to work with ribosome encodings

use holochain_core_types::{
    bits_n_pieces::{u64_merge_bits, u64_split_bits},
    error::{
        HolochainError, RibosomeEncodedAllocation, RibosomeReturnValue, WasmAllocationInt,
        RibosomeErrorCode,
    },
};
use memory::handler::WasmMemoryHandler;
// use memory::handler::WasmMemoryHandler;

use holochain_json_api::json::JsonString;

use memory::{
    allocation::{AllocationError, AllocationResult, WasmAllocation},
    MemoryBits,
};
use std::convert::TryFrom;

impl TryFrom<RibosomeEncodedAllocation> for WasmAllocation {
    type Error = AllocationError;
    fn try_from(
        ribosome_memory_allocation: RibosomeEncodedAllocation,
    ) -> Result<Self, Self::Error> {
        let (offset, length) = u64_split_bits(MemoryBits::from(ribosome_memory_allocation));
        WasmAllocation::new(offset.into(), length.into())
    }
}

impl From<WasmAllocation> for RibosomeEncodedAllocation {
    fn from(wasm_allocation: WasmAllocation) -> Self {
        u64_merge_bits(
            wasm_allocation.offset().into(),
            wasm_allocation.length().into(),
        )
        .into()
    }
}

impl From<WasmAllocation> for RibosomeReturnValue {
    fn from(wasm_allocation: WasmAllocation) -> Self {
        RibosomeReturnValue::Allocation(RibosomeEncodedAllocation::from(wasm_allocation))
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

impl From<AllocationError> for RibosomeReturnValue {
    fn from(allocation_error: AllocationError) -> Self {
        RibosomeReturnValue::Failure(RibosomeErrorCode::from(allocation_error))
    }
}

impl AllocationError {
    pub fn as_ribosome_encoding(&self) -> WasmAllocationInt {
        RibosomeReturnValue::from(self.clone()).into()
    }
}

impl TryFrom<WasmAllocationInt> for WasmAllocation {

}

impl WasmAllocation {
    /// equivalent to TryFrom<WasmAllocationInt> for WasmAllocation
    /// not implemented as a trait because WasmAllocationInt is a primitive and that would couple
    /// allocations to ribosome encoding
    pub fn try_from_ribosome_encoding(allocation_int: WasmAllocationInt) -> AllocationResult {
        match RibosomeReturnValue::from(allocation_int) {
            RibosomeReturnValue::Success => Err(AllocationError::ZeroLength),
            RibosomeReturnValue::Failure(_) => Err(AllocationError::OutOfBounds),
            RibosomeReturnValue::Allocation(ribosome_allocation) => {
                WasmAllocation::try_from(ribosome_allocation)
            }
        }
    }

    pub fn as_ribosome_encoding(self) -> WasmAllocationInt {
        RibosomeReturnValue::from(self).into()
    }
}

// impl WasmStack {
//     /// equivalent to TryFrom<WasmAllocationInt> for WasmStack
//     /// not implemented as a trait because WasmAllocationInt is a primitive and that would couple
//     /// stacks to ribosome encoding
//     /// wraps WasmAllocation::try_from_ribosome_encoding internally but has a "higher level"
//     /// return signature intended for direct use/return in/from ribosome fns
//     pub fn try_from_ribosome_encoding(
//         maybe_encoded_allocation: WasmAllocationInt,
//     ) -> Result<WasmStack, RibosomeReturnValue> {
//         match WasmAllocation::try_from_ribosome_encoding(maybe_encoded_allocation) {
//             Err(allocation_error) => Err(allocation_error),
//             Ok(allocation) => match WasmStack::try_from(allocation) {
//                 Err(allocation_error) => Err(allocation_error),
//                 Ok(stack) => Ok(stack),
//             },
//         }
//         .map_err(|e| e.as_ribosome_encoding().into())
//     }
// }

/// Equivalent to From<AllocationResult> for RibosomeReturnValue
/// not possible to implement the trait as Result and RibosomeReturnValue from different crates
pub fn return_code_for_allocation_result(result: AllocationResult) -> RibosomeReturnValue {
    match result {
        Ok(allocation) => RibosomeReturnValue::from(allocation),
        Err(allocation_error) => RibosomeReturnValue::from(allocation_error),
    }
}

pub fn load_ribosome_encoded_string<W: WasmMemoryHandler>(
    wasm_memory_handler: &W,
    encoded_value: WasmAllocationInt,
) -> Result<String, HolochainError> {
    // almost the same as WasmAllocation::try_from_ribosome_encoding but maps to HolochainError
    match RibosomeReturnValue::from(encoded_value) {
        RibosomeReturnValue::Success => Err(HolochainError::Ribosome(
            RibosomeErrorCode::ZeroSizedAllocation,
        )),
        RibosomeReturnValue::Failure(err_code) => Err(HolochainError::Ribosome(err_code)),
        RibosomeReturnValue::Allocation(ribosome_allocation) => {
            Ok(wasm_memory_handler.read_string(WasmAllocation::try_from(ribosome_allocation)?))
        }
    }
}

pub fn load_ribosome_encoded_json<W: WasmMemoryHandler, J: TryFrom<JsonString>>(
    wasm_memory_handler: &W,
    encoded_value: WasmAllocationInt,
) -> Result<J, HolochainError>
where
    J::Error: Into<HolochainError>,
{
    let s = load_ribosome_encoded_string(wasm_memory_handler, encoded_value)?;
    let j = JsonString::from_json(&s);

    J::try_from(j).map_err(|e| e.into())
}

#[cfg(test)]
pub mod tests {

    use holochain_core_types::{
        bits_n_pieces::u64_merge_bits,
        error::{
            RibosomeEncodedAllocation, RibosomeReturnValue, WasmAllocationInt,
            RibosomeErrorCode,
        },
    };
    use memory::{
        allocation::{AllocationError, Length, Offset, WasmAllocation},
        ribosome::return_code_for_allocation_result,
        stack::{Top, WasmStack},
    };
    use std::convert::TryFrom;

    #[test]
    fn try_allocation_from_ribosome_allocation_test() {
        assert_eq!(
            Err(AllocationError::ZeroLength),
            WasmAllocation::try_from(RibosomeEncodedAllocation::from(0_u64)),
        );

        assert_eq!(
            Err(AllocationError::OutOfBounds),
            WasmAllocation::try_from(RibosomeEncodedAllocation::from(u64_merge_bits(
                std::u32::MAX,
                std::u32::MAX
            ))),
        );

        assert_eq!(
            Ok(WasmAllocation {
                offset: Offset::from(4_u32),
                length: Length::from(8_u32)
            }),
            WasmAllocation::try_from(RibosomeEncodedAllocation::from(
                0b000000000000000000000000000000100_00000000000000000000000000001000
            )),
        );
    }

    #[test]
    fn ribosome_allocation_from_allocation_test() {
        assert_eq!(
            RibosomeEncodedAllocation::from(
                0b00000000000000000000000000000100_00000000000000000000000000001000
            ),
            RibosomeEncodedAllocation::from(WasmAllocation {
                offset: Offset::from(4_u32),
                length: Length::from(8_u32)
            }),
        );
    }

    #[test]
    fn ribosome_encoded_value_from_allocation_test() {
        assert_eq!(
            RibosomeReturnValue::Allocation(RibosomeEncodedAllocation::from(
                0b00000000000000000000000000000100_00000000000000000000000000001000
            )),
            RibosomeReturnValue::from(WasmAllocation {
                offset: Offset::from(4_u32),
                length: Length::from(8_u32)
            }),
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
            RibosomeReturnValue::Failure(RibosomeErrorCode::OutOfMemory),
            RibosomeReturnValue::from(AllocationError::OutOfBounds),
        );

        assert_eq!(
            RibosomeReturnValue::Failure(RibosomeErrorCode::ZeroSizedAllocation),
            RibosomeReturnValue::from(AllocationError::ZeroLength),
        );

        assert_eq!(
            RibosomeReturnValue::Failure(RibosomeErrorCode::NotAnAllocation),
            RibosomeReturnValue::from(AllocationError::BadStackAlignment),
        );

        assert_eq!(
            RibosomeReturnValue::Failure(RibosomeErrorCode::NotAnAllocation),
            RibosomeReturnValue::from(AllocationError::Serialization),
        );
    }

    #[test]
    fn ribosome_encoding_test() {
        assert_eq!(
            WasmAllocationInt::from(RibosomeReturnValue::Failure(
                RibosomeErrorCode::OutOfMemory
            )),
            AllocationError::OutOfBounds.as_ribosome_encoding(),
        );
        assert_eq!(
            WasmAllocationInt::from(RibosomeReturnValue::Failure(
                RibosomeErrorCode::ZeroSizedAllocation
            )),
            AllocationError::ZeroLength.as_ribosome_encoding(),
        );
        assert_eq!(
            WasmAllocationInt::from(RibosomeReturnValue::Failure(
                RibosomeErrorCode::NotAnAllocation
            )),
            AllocationError::BadStackAlignment.as_ribosome_encoding(),
        );
        assert_eq!(
            WasmAllocationInt::from(RibosomeReturnValue::Failure(
                RibosomeErrorCode::NotAnAllocation
            )),
            AllocationError::Serialization.as_ribosome_encoding(),
        );
    }

    #[test]
    fn stack_from_encoding_test() {
        assert_eq!(
            Err(RibosomeReturnValue::from(AllocationError::OutOfBounds)),
            WasmStack::try_from_ribosome_encoding(u64_merge_bits(std::u32::MAX, std::u32::MAX)),
        );

        assert_eq!(
            Err(RibosomeReturnValue::from(AllocationError::ZeroLength)),
            WasmStack::try_from_ribosome_encoding(0),
        );

        assert_eq!(
            Ok(WasmStack { top: Top(4) }),
            // 2 + 2 = 4
            WasmStack::try_from_ribosome_encoding(
                0b00000000000000000000000000000010_00000000000000000000000000000010
            ),
        );
    }

    #[test]
    fn return_code_for_allocation_result_test() {
        assert_eq!(
            RibosomeReturnValue::from(AllocationError::OutOfBounds),
            return_code_for_allocation_result(Err(AllocationError::OutOfBounds)),
        );
        assert_eq!(
            RibosomeReturnValue::from(AllocationError::ZeroLength),
            return_code_for_allocation_result(Err(AllocationError::ZeroLength)),
        );
        assert_eq!(
            RibosomeReturnValue::from(AllocationError::BadStackAlignment),
            return_code_for_allocation_result(Err(AllocationError::BadStackAlignment)),
        );
        assert_eq!(
            RibosomeReturnValue::from(AllocationError::Serialization),
            return_code_for_allocation_result(Err(AllocationError::Serialization)),
        );
        let allocation = WasmAllocation {
            offset: Offset::from(5_u32),
            length: Length::from(5_u32),
        };
        assert_eq!(
            RibosomeReturnValue::from(allocation),
            return_code_for_allocation_result(Ok(allocation)),
        );
    }
}
