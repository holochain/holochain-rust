use super::Dispatch;
use error::ZomeApiResult;
use holochain_core_types::time::Timeout;
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::send::{SendArgs, SendOptions};

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
/// # use holochain_core_types::error::RibosomeEncodingBits;
/// # use holochain_core_types::error::RibosomeEncodedValue;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_crypto(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_meta(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sign_one_time(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_verify_signature(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// /// # #[no_mangle]
/// # pub fn hc_encrypt(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_sleep(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_debug(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_links(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_links_count(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_link_entries(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_remove_link(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_list(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_new_random(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_seed(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_derive_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_keystore_get_public_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_commit_capability_grant(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_commit_capability_claim(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_emit_signal(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
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
pub fn send(to_agent: Address, payload: String, timeout: Timeout) -> ZomeApiResult<String> {
    Dispatch::Send.with_input(SendArgs {
        to_agent,
        payload,
        options: SendOptions(timeout),
    })
}
