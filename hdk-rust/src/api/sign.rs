use error::ZomeApiResult;
use holochain_core_types::signature::Provenance;
use holochain_wasm_utils::api_serialization::{
    crypto::{CryptoArgs, CryptoMethod},
    sign::{OneTimeSignArgs, SignOneTimeResult},
    verify_signature::VerifySignatureArgs,
};

use super::Dispatch;

/// Signs a string payload using the agent's private key.
/// Returns the signature as a string.
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # extern crate holochain_persistence_api;
/// # extern crate holochain_json_api;
/// # #[macro_use]
/// # extern crate holochain_json_derive;
/// # use holochain_json_api::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::signature::{Provenance, Signature};
/// # use hdk::error::ZomeApiResult;
/// # fn main() {
/// pub fn handle_sign_message(message: String) -> ZomeApiResult<Signature> {
///    hdk::sign(message).map(Signature::from)
/// }
/// # }
/// ```
pub fn sign<S: Into<String>>(payload: S) -> ZomeApiResult<String> {
    Dispatch::Crypto.with_input(CryptoArgs {
        payload: payload.into(),
        method: CryptoMethod::Sign,
    })
}

/// Signs a vector of payloads with a private key that is generated and shredded.
/// Returns the signatures of the payloads and the public key that can be used to verify the signatures.
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # extern crate holochain_persistence_api;
/// # extern crate holochain_json_api;
/// # #[macro_use]
/// # extern crate holochain_json_derive;
/// # use holochain_json_api::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::signature::{Provenance, Signature};
/// # use hdk::error::ZomeApiResult;
/// # use hdk::holochain_wasm_utils::api_serialization::sign::{OneTimeSignArgs, SignOneTimeResult};
/// # fn main() {
/// pub fn handle_one_time_sign(key_id: String, message: String) -> ZomeApiResult<Signature> {
///    hdk::sign(message).map(Signature::from)
/// }
/// # }
/// ```
pub fn sign_one_time<S: Into<String>>(payloads: Vec<S>) -> ZomeApiResult<SignOneTimeResult> {
    let mut converted_payloads = Vec::new();
    for p in payloads {
        converted_payloads.push(p.into());
    }
    Dispatch::SignOneTime.with_input(OneTimeSignArgs {
        payloads: converted_payloads,
    })
}

/// Verifies a provenance (public key, signature) against a payload
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # extern crate holochain_persistence_api;
/// # extern crate holochain_json_api;
/// # #[macro_use]
/// # extern crate holochain_json_derive;
/// # use holochain_json_api::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::signature::Provenance;
/// # use hdk::error::ZomeApiResult;
/// # fn main() {
/// pub fn handle_verify_message(message: String, provenance: Provenance) -> ZomeApiResult<bool> {
///     hdk::verify_signature(provenance, message)
/// }
/// # }
/// ```
pub fn verify_signature<S: Into<String>>(
    provenance: Provenance,
    payload: S,
) -> ZomeApiResult<bool> {
    Dispatch::VerifySignature.with_input(VerifySignatureArgs {
        provenance,
        payload: payload.into(),
    })
}
