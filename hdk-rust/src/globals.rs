//! Holds the internal/private globals used by the zome api library.
//! Also contains the functions declarations of the external functions provided by the Ribosome.

use crate::init_globals::init_globals;
use holochain_core_types::error::RibosomeEncodingBits;
use holochain_wasm_utils::{api_serialization::ZomeApiGlobals, memory::stack::WasmStack};

/// Internal global for memory usage
pub static mut G_MEM_STACK: Option<WasmStack> = None;

// Internal global for retrieving all Zome API globals
lazy_static! {
    pub(crate) static ref GLOBALS: ZomeApiGlobals = init_globals().unwrap();
}

// Invokable functions in the Ribosome
// WARNING Names must be in sync with ZomeAPIFunction in holochain-rust
// WARNING All these fns need to be defined in wasms too @see the hdk integration_test.rs
#[allow(dead_code)]
extern "C" {
    pub(crate) fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_property(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_debug(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_sign_one_time(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_verify_signature(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_link_entries(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_get_links(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_start_bundle(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_close_bundle(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_sleep(_: RibosomeEncodingBits) -> RibosomeEncodingBits;

    pub(crate) fn hc_remove_link(_: RibosomeEncodingBits) -> RibosomeEncodingBits;
}
