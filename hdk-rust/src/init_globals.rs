//! Holds the internal/private zome API function `init_globals`
//! which initializes the Zome API Globals with the values it receives from the Ribosome.
//! It is automatically called at startup of each Zome function call.

use holochain_wasm_utils::{api_serialization::ZomeApiGlobals, memory_serialization::load_json};

extern "C" {
    fn hc_init_globals(encoded_allocation_of_input: u32) -> u32;
}

// HC INIT GLOBALS - Secret Api Function
// Retrieve all the public global values from the ribosome
pub(crate) fn init_globals() -> ZomeApiGlobals {
    // Call WASMI-able init_globals
    let encoded_allocation_of_result = unsafe { hc_init_globals(0) };
    // Deserialize complex result stored in memory
    let result = load_json(encoded_allocation_of_result as u32);
    if result.is_err() {
        panic!("ZomeApiGlobals should deserialize properly");
    }
    result.unwrap()
}
