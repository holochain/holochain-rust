//! File for holding all internal/private globals used by the zome api library

use holochain_wasm_utils::memory_allocation::SinglePageStack;
use init_globals::init_globals;
use init_globals::AppGlobals;

// Internal global for memory usage
pub static mut G_MEM_STACK: Option<SinglePageStack> = None;

// Internal global for retrieving all app globals
lazy_static! {
    pub(crate) static ref APP_GLOBALS: AppGlobals = init_globals();
}

// Invokable functions in the ribosome
// WARNING Names must be in sync with ZomeAPIFunction in holochain-rust
extern "C" {
    pub(crate) fn hc_property(encoded_allocation_of_input: u32) -> u32;
    pub(crate) fn hc_make_hash(encoded_allocation_of_input: u32) -> u32;
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
