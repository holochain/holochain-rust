use super::Dispatch;
use error::ZomeApiResult;
use holochain_json_api::json::JsonString;
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::ZomeFnCallArgs;

/// Call an exposed function from another zome or another (bridged) instance running
/// in the same conductor.
/// Arguments for the called function are passed and resturned as `JsonString`.
/// # Examples
/// Here are two example Zomes, where one performs a `call` into the other.
///
/// This first zome is the "callee"; i.e., the zome that receives the call, and is named `summer`.
/// because the call sums two numbers.
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # use hdk::holochain_json_api::json::JsonString;
/// # use hdk::holochain_json_api::error::JsonError;
/// # use hdk::holochain_core_types::error::HolochainError;
/// # use hdk::holochain_core_types::error::RibosomeEncodingBits;
/// # use hdk::holochain_core_types::error::RibosomeEncodedValue;
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
/// #[no_mangle]
/// # pub fn hc_crypto(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// #[no_mangle]
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
///
/// fn handle_sum(num1: u32, num2: u32) -> JsonString {
///     let sum = num1 + num2;
///     json!({"sum": sum.to_string()}).into()
/// }
///
/// define_zome! {
///     entries: []
///
///     init: || {
///         Ok(())
///     }
///
///     validate_agent: |validation_data : EntryValidationData::<AgentId>| {
///         Ok(())
///     }
///     
///     functions: [
///             sum: {
///                 inputs: |num1: u32, num2: u32|,
///                 outputs: |sum: JsonString|,
///                 handler: handle_sum
///             }
///     ]
///
///     traits: {
///         hc_public [sum]
///     }
/// }
///
/// # }
/// ```
///
/// This second zome is the "caller" that makes the call into the `summer` Zome.
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate holochain_json_derive;
///
/// # use hdk::holochain_persistence_api::hash::HashString;
/// # use hdk::holochain_json_api::error::JsonError;
/// # use hdk::holochain_json_api::json::JsonString;
/// # use hdk::holochain_core_types::error::HolochainError;
/// # use hdk::error::ZomeApiResult;
/// # use std::convert::TryInto;
/// # use hdk::holochain_core_types::error::RibosomeEncodingBits;
/// # use hdk::holochain_core_types::error::RibosomeEncodedValue;
/// # use hdk::holochain_persistence_api::cas::content::Address;
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
/// # #[no_mangle]
/// # pub fn hc_sleep(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_debug(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// # #[no_mangle]
/// # pub fn hc_get_links(_: RibosomeEncodingBits) -> RibosomeEncodingBits { RibosomeEncodedValue::Success.into() }
/// #[no_mangle]
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
///
/// fn handle_check_sum(num1: u32, num2: u32) -> ZomeApiResult<JsonString> {
///     #[derive(Serialize, Deserialize, Debug, DefaultJson)]
///     struct SumInput {
///         num1: u32,
///         num2: u32,
///     };
///     let call_input = SumInput {
///         num1: num1,
///         num2: num2,
///     };
///     hdk::call(hdk::THIS_INSTANCE, "summer", Address::from(hdk::PUBLIC_TOKEN.to_string()), "sum", call_input.into())
/// }
///
/// define_zome! {
///     entries: []
///
///     init: || {
///         Ok(())
///     }
///     
///     validate_agent: |validation_data : EntryValidationData::<AgentId>| {
///         Ok(())
///     }
///
///     functions: [
///             check_sum: {
///                 inputs: |num1: u32, num2: u32|,
///                 outputs: |sum: ZomeApiResult<JsonString>|,
///                 handler: handle_check_sum
///             }
///     ]
///
///     traits: {
///         hc_public [check_sum]
///     }
/// }
///
/// # }
/// ```
pub fn call<S: Into<String>>(
    instance_handle: S,
    zome_name: S,
    cap_token: Address,
    fn_name: S,
    fn_args: JsonString,
) -> ZomeApiResult<JsonString> {
    Dispatch::Call.with_input(ZomeFnCallArgs {
        instance_handle: instance_handle.into(),
        zome_name: zome_name.into(),
        cap_token,
        fn_name: fn_name.into(),
        fn_args: String::from(fn_args),
    })
}
