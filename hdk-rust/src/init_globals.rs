//! Holds the internal/private zome API function `init_globals`
//! which initializes the Zome API Globals with the values it receives from the Ribosome.
//! It is automatically called at startup of each Zome function call.

use error::{ZomeApiError, ZomeApiResult};
use holochain_core_types::{error::ZomeApiInternalResult, json::JsonString};
use holochain_wasm_utils::{api_serialization::ZomeApiGlobals};
use std::convert::TryInto;
use holochain_wasm_utils::memory::allocation::WasmAllocation;
use holochain_core_types::error::RibosomeEncodedAllocation;
use holochain_wasm_utils::memory::MemoryBits;

#[allow(dead_code)]
extern "C" {
    fn hc_init_globals(encoded_allocation_of_input: MemoryBits) -> MemoryBits;
}

// HC INIT GLOBALS - Secret Api Function
// Retrieve all the public global values from the ribosome
pub(crate) fn init_globals() -> ZomeApiResult<ZomeApiGlobals> {
    // Call WASMI-able init_globals
    let encoded_allocation_of_result = unsafe { hc_init_globals(0) };
    let wasm_allocation = WasmAllocation::from(RibosomeEncodedAllocation::from(encoded_allocation_of_result));
    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = wasm_allocation.read_json()?;
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
}
