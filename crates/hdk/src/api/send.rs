use crate::{error::ZomeApiResult};
use holochain_core_types::time::Timeout;
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_types::send::{SendArgs, SendOptions};
use crate::api::DNA_NAME;
use holochain_wasmer_guest::host_call;
use crate::api::hc_send;
use holochain_json_api::json::RawString;

/// Sends a node-to-node message to the given agent, specified by their address.
/// Addresses of agents can be accessed using [hdk::AGENT_ADDRESS](struct.AGENT_ADDRESS.html).
/// This works in conjunction with the `receive` callback that has to be defined in the
/// [define_zome!](../macro.define_zome.html) macro.
///
/// This function dispatches a message to the receiver, and will wait up to 60 seconds before returning a timeout error. The `send` function will return the string returned
/// by the `receive` callback of the other node.
/// # Examples
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # extern crate holochain_persistence_api;
/// # extern crate holochain_json_api;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # use hdk::error::ZomeApiResult;
/// # use holochain_persistence_api::cas::content::Address;
/// # use holochain_json_api::error::JsonError;
/// # use holochain_json_api::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::error::AllocationPtr;
/// # use holochain_core_types::error::RibosomeReturnValue;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_query(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_call(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_crypto(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_meta(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sign_one_time(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_verify_signature(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_send(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// /// # #[no_mangle]
/// # pub fn hc_encrypt(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sleep(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_debug(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_links(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_links_count(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_link_entries(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_link(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_list(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_new_random(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_seed(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_key(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_sign(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_get_public_key(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_commit_capability_grant(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_commit_capability_claim(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_emit_signal(_: AllocationPtr) -> AllocationPtr { RibosomeReturnValue::Success.into() }
///
/// # fn main() {
/// fn handle_send_message(to_agent: Address, message: String) -> ZomeApiResult<String> {
///     // because the function signature of hdk::send is the same as the
///     // signature of handle_send_message we can just directly return its' result
///     hdk::send(to_agent, message, 60000.into())
/// }
///
/// define_zome! {
///    entries: []
///
///    init: || { Ok(()) }
///
///    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
///        Ok(())
///    }
///
///    receive: |from, payload| {
///        // if you want to serialize data as json to pass, use the json! serde macro
///        json!({
///            "key": "value"
///        }).to_string()
///    }
///
///    functions: [
///            send_message: {
///                inputs: |to_agent: Address, message: String|,
///                outputs: |response: ZomeApiResult<String>|,
///                handler: handle_send_message
///            }
///    ]
///
///     traits: {
///         hc_public [send_message]
///     }
///}
/// # }
/// ```
pub fn send(to_agent: Address, payload: String, timeout: Timeout) -> ZomeApiResult<RawString> {
    Ok(host_call!(hc_send, SendArgs {
        to_agent,
        payload,
        options: SendOptions(timeout),
        zome: DNA_NAME.to_string(),
    })?)
}
