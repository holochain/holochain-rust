//! Holds the internal/private zome API function `init_globals`
//! which initializes the Zome API Globals with the values it receives from the Ribosome.
//! It is automatically called at startup of each Zome function call.

use crate::{error::ZomeApiResult};
use holochain_wasm_types::ZomeApiGlobals;
use holochain_wasmer_guest::AllocationPtr;
use holochain_wasmer_guest::host_call;
// use crate::debug;

#[allow(dead_code)]
extern "C" {
    pub fn hc_init_globals(_: AllocationPtr) -> AllocationPtr;
}

// HC INIT GLOBALS - Secret Api Function
// Retrieve all the public global values from the ribosome
pub(crate) fn init_globals() -> ZomeApiResult<ZomeApiGlobals> {
    // let r = host_call!(hc_init_globals, ());
    // debug(format!("{:?}", r)).ok();
    // Ok(r?)
    Ok(host_call!(hc_init_globals, ())?)
}
