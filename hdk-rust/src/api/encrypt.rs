use error::ZomeApiResult;
use holochain_wasm_utils::api_serialization::crypto::{CryptoArgs, CryptoMethod};

use super::Dispatch;

/// encrypts a string payload using the agent's private key.
/// Returns the message as a string.
/// # Examples
/// ```rust
/// # #![feature(try_from)]
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # #[macro_use]
/// # extern crate holochain_json_derive;
/// # use hdk::holochain_json_api::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::signature::{Provenance, Signature};
/// # use hdk::error::ZomeApiResult;
/// # fn main() {
/// pub fn handle_encrypt_message(message: String) -> ZomeApiResult<Signature> {
///    hdk::encrypt(message).map(Signature::from)
/// }
/// # }
/// ```
pub fn encrypt<S: Into<String>>(payload: S) -> ZomeApiResult<String> {
    Dispatch::Crypto.with_input(CryptoArgs {
        payload: payload.into(),
        method: CryptoMethod::Encrypt,
    })
}
