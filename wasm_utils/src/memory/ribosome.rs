// extends memory allocation to work with ribosome encodings

use memory::MemoryBits;
use memory::allocation::WasmAllocation;
use holochain_core_types::error::RibosomeReturnCode;
use holochain_core_types::error::RibosomeEncodedAllocation;
use holochain_core_types::error::RibosomeErrorCode;
use memory::allocation::AllocationError;
use std::convert::TryFrom;
use holochain_core_types::bits_n_pieces::u32_split_bits;
use holochain_core_types::bits_n_pieces::u32_merge_bits;
use holochain_core_types::error::HolochainError;
use holochain_core_types::json::JsonString;
use holochain_core_types::error::RibosomeEncodingBits;

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

pub fn return_code_for_allocation_result(result: Result<WasmAllocation, AllocationError>) -> RibosomeReturnCode {
    match result {
        Ok(allocation) => RibosomeReturnCode::from(allocation),
        Err(allocation_error) => RibosomeReturnCode::from(allocation_error),
    }
}

pub fn load_ribosome_encoded_json<J: TryFrom<JsonString>>(encoded_value: RibosomeEncodingBits) -> Result<J, HolochainError>
    where J::Error: Into<HolochainError>{

    match RibosomeReturnCode::from(encoded_value) {
        RibosomeReturnCode::Success => Err(HolochainError::Ribosome(RibosomeErrorCode::ZeroSizedAllocation))?,
        RibosomeReturnCode::Failure(err_code) => Err(HolochainError::Ribosome(err_code))?,
        RibosomeReturnCode::Allocation(ribosome_allocation) => {

            let allocation = WasmAllocation::try_from(ribosome_allocation)?;
            let s = allocation.read_to_string();
            let j = JsonString::from(s);

            J::try_from(j).map_err(|e| e.into() )

        }
    }

}
