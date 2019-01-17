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

impl From<AllocationError> for RibosomeEncodedValue {
    fn from(allocation_error: AllocationError) -> Self {
        RibosomeEncodedValue::Failure(RibosomeErrorCode::from(allocation_error))
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
#[cfg(feature = "broken-tests")]
pub mod tests {

    use holochain_core_types::error::{
        RibosomeEncodedValue, RibosomeEncodingBits, RibosomeErrorCode,
    };

    #[test]
    fn ribosome_return_code_round_trip() {
        let oom = RibosomeEncodedValue::from(
            (RibosomeErrorCode::OutOfMemory as RibosomeEncodingBits) >> 16,
        );
        assert_eq!(
            RibosomeEncodedValue::Failure(RibosomeErrorCode::OutOfMemory),
            oom
        );
        assert_eq!(RibosomeErrorCode::OutOfMemory.to_string(), oom.to_string());
    }

}
