//! Holds the internal/private zome API function `init_globals`
//! which initializes the Zome API Globals with the values it receives from the Ribosome.
//! It is automatically called at startup of each Zome function call.

use crate::api::Dispatch;
use error::ZomeApiResult;
use holochain_core_types::error::RibosomeEncodingBits;
use holochain_wasm_utils::api_serialization::ZomeApiGlobals;

#[allow(dead_code)]
extern "C" {
    pub fn hc_init_globals(
        encoded_allocation_of_input: RibosomeEncodingBits,
    ) -> RibosomeEncodingBits;
}

// HC INIT GLOBALS - Secret Api Function
// Retrieve all the public global values from the ribosome
pub(crate) fn init_globals() -> ZomeApiResult<ZomeApiGlobals> {
    Dispatch::InitGlobals.with_input(0)
}
