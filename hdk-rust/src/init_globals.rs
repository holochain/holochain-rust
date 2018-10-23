//! Holds the internal/private zome API function `init_globals`
//! which initializes into WASM memory the values it receives
//! from the DNA, by calling the Zome once.

use holochain_wasm_utils::{
    holochain_core_types::{
        app_globals::AppGlobals,
        hash::HashString
    },
    memory_serialization::load_json,
};

extern "C" {
    fn hc_init_globals(encoded_allocation_of_input: u32) -> u32;
}

// HC INIT GLOBALS - Secret Api Function
// Retrieve all the public global values from the ribosome
pub(crate) fn init_globals() -> AppGlobals {
    // Call WASMI-able init_globals
    let encoded_allocation_of_result = unsafe { hc_init_globals(0) };
    // Deserialize complex result stored in memory
    let result = load_json(encoded_allocation_of_result as u32);
    if result.is_err() {
        panic!("AppGlobals should deserialize properly");
    }
    result.unwrap()
}
