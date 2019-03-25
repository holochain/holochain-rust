#![feature(try_from)]

#[macro_use]
extern crate hdk;
#[macro_use]
extern crate serde_derive;

use hdk::{
    error::ZomeApiResult,
    holochain_core_types::{
        error::HolochainError,
        json::JsonString,
        signature::{Provenance, Signature},
    },
    holochain_wasm_utils::api_serialization::keystore::KeyType,
};

pub fn handle_sign_message(key_id: String, message: String) -> ZomeApiResult<Signature> {
    if key_id == "" {
        hdk::sign(message).map(Signature::from)
    } else {
        hdk::keystore_sign(key_id, message).map(Signature::from)
    }
}

pub fn handle_verify_message(message: String, provenance: Provenance) -> ZomeApiResult<bool> {
    hdk::verify_signature(provenance, message)
}

pub fn handle_add_key(id: String) -> ZomeApiResult<String> {
    hdk::keystore_derive_key("app_seed", &id, KeyType::Signing)
}

pub fn handle_list_secrets() -> ZomeApiResult<Vec<String>> {
    hdk::keystore_list().map(|keystore_ids| keystore_ids.ids)
}

define_zome! {
    entries: []

    genesis: || {
        {
            hdk::keystore_new_random("app_seed", 32)
                .map_err(|err|
                         format!("new seed generation failed: {}",err)
            )
        }
    }

    functions: [
        sign_message: {
            inputs: |key_id: String, message: String|,
            outputs: |result: ZomeApiResult<Signature>|,
            handler: handle_sign_message
        }

        verify_message: {
            inputs: |message: String, provenance: Provenance|,
            outputs: |result: ZomeApiResult<bool>|,
            handler: handle_verify_message
        }

        add_key: {
            inputs: |id: String|,
            outputs: |result: ZomeApiResult<String>|,
            handler: handle_add_key
        }

        list_secrets: {
            inputs: | |,
            outputs: |result: ZomeApiResult<Vec<String>>|,
            handler: handle_list_secrets
        }

    ]

    traits: {
        hc_public [sign_message, verify_message, add_key, list_secrets]
    }
}
