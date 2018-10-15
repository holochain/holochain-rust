//! File for holding the internal/private zome api function `init_globals`

use holochain_wasm_utils::{
    holochain_core_types::hash::HashString,
    memory_serialization::try_deserialize_allocation,
};

extern "C" {
    fn hc_init_globals(encoded_allocation_of_input: u32) -> u32;
}

// WARNING must be in sync with InitGlobalsOutput in core
#[derive(Deserialize, Clone)]
pub(crate) struct AppGlobals {
    pub app_name: String,
    pub app_dna_hash: HashString,
    pub app_agent_id_str: String,
    pub app_agent_key_hash: HashString,
    pub app_agent_initial_hash: HashString,
    pub app_agent_latest_hash: HashString,
}

// HC INIT GLOBALS - Secret Api Function
// Retrieve all the public global values from the ribosome
pub(crate) fn init_globals() -> AppGlobals {
    // Call WASMI-able init_globals
    let encoded_allocation_of_result = unsafe { hc_init_globals(0) };
    // Deserialize complex result stored in memory
    let result = try_deserialize_allocation(encoded_allocation_of_result as u32);
    if result.is_err() {
        panic!("AppGlobals should deserialize properly");
    }
    result.unwrap()
}
