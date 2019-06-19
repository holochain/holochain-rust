use error::ZomeApiResult;
use holochain_wasm_utils::api_serialization::{
    crypto::{CryptoArgs,ConductorCryptoApiMethod}
};


use super::Dispatch;

/// Signs a string payload using the agent's private key.
/// Returns the signature as a string.
/// # Examples
/// ```rust
/// # #![feature(try_from)]
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # #[macro_use]
/// # extern crate holochain_core_types_derive;
/// # use holochain_core_types::json::JsonString;
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
        method : ConductorCryptoApiMethod::Encrypt
    })
}



