use crate::{error::ZomeApiResult};
use holochain_wasm_types::keystore::{
    KeyType, KeystoreDeriveKeyArgs, KeystoreDeriveSeedArgs, KeystoreGetPublicKeyArgs,
    KeystoreListResult, KeystoreNewRandomArgs, KeystoreSignArgs,
};
use holochain_wasmer_guest::host_call;
use crate::api::hc_keystore_get_public_key;
use crate::api::hc_keystore_sign;
use crate::api::hc_keystore_derive_key;
use crate::api::hc_keystore_derive_seed;
use crate::api::hc_keystore_new_random;
use crate::api::hc_keystore_list;

// Returns a list of the named secrets stored in the keystore.
pub fn keystore_list() -> ZomeApiResult<KeystoreListResult> {
    host_call!(hc_keystore_list, ())?
}

/// Creates a new random "root" Seed secret in the keystore
pub fn keystore_new_random<S: Into<String>>(dst_id: S, size: usize) -> ZomeApiResult<()> {
    host_call!(hc_keystore_new_random, KeystoreNewRandomArgs {
        dst_id: dst_id.into(),
        size,
    })?
}

/// Creates a new derived seed secret in the keystore, derived from a previously defined seed.
/// Accepts two arguments: the keystore ID of the previously defined seed, and a keystore ID for the newly derived seed.
pub fn keystore_derive_seed<S: Into<String>>(
    src_id: S,
    dst_id: S,
    context: S,
    index: u64,
) -> ZomeApiResult<()> {
    host_call!(hc_keystore_derive_seed, KeystoreDeriveSeedArgs {
        src_id: src_id.into(),
        dst_id: dst_id.into(),
        context: context.into(),
        index,
    })?
}

/// Creates a new derived key secret in the keystore derived from on a previously defined seed.
/// Accepts two arguments: the keystore ID of the previously defined seed, and a keystore ID for the newly derived key.
pub fn keystore_derive_key<S: Into<String>>(
    src_id: S,
    dst_id: S,
    key_type: KeyType,
) -> ZomeApiResult<String> {
    host_call!(hc_keystore_derive_key, KeystoreDeriveKeyArgs {
        src_id: src_id.into(),
        dst_id: dst_id.into(),
        key_type,
    })?
}

/// Signs a payload using a private key from the keystore.
/// Accepts one argument: the keystore ID of the desired private key.
pub fn keystore_sign<S: Into<String>>(src_id: S, payload: S) -> ZomeApiResult<String> {
    host_call!(hc_keystore_sign, KeystoreSignArgs {
        src_id: src_id.into(),
        payload: payload.into(),
    })?
}

/// Returns the public key of a key secret
/// Accepts one argument: the keystore ID of the desired public key.
/// Fails if the id is a Seed secret.
pub fn keystore_get_public_key<S: Into<String>>(src_id: S) -> ZomeApiResult<String> {
    host_call!(hc_keystore_get_public_key, KeystoreGetPublicKeyArgs {
        src_id: src_id.into(),
    })?
}
