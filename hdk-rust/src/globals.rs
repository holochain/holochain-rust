//! Holds the internal/private globals used by the zome api library.
//! Also contains the functions declarations of the external functions provided by the Ribosome.

use crate::init_globals::init_globals;
use holochain_wasm_utils::{api_serialization::ZomeApiGlobals};

/// Internal global for memory usage
pub static mut G_MEM_STACK: Option<SinglePageStack> = None;

// Internal global for retrieving all Zome API globals
lazy_static! {
    pub(crate) static ref GLOBALS: ZomeApiGlobals = init_globals().unwrap();
}

// Invokable functions in the Ribosome
// WARNING Names must be in sync with ZomeAPIFunction in holochain-rust
#[allow(dead_code)]
extern "C" {
    pub(crate) fn hc_property(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_entry_address(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_debug(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_call(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_sign(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_verify_signature(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_commit_entry(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_update_entry(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_remove_entry(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_get_entry(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_link_entries(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_get_links(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_query(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_send(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_start_bundle(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_close_bundle(encoded_allocation_of_input: u32) -> u32;
}
