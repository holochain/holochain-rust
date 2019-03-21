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
};

pub fn handle_sign_message(message: String) -> ZomeApiResult<Signature> {
    hdk::sign(message).map(Signature::from)
}

pub fn handle_verify_message(message: String, provenance: Provenance) -> ZomeApiResult<bool> {
    hdk::verify_signature(provenance, message)
}

define_zome! {
    entries: []

    genesis: || { Ok(()) }

    functions: [
        sign_message: {
            inputs: |message: String|,
            outputs: |result: ZomeApiResult<Signature>|,
            handler: handle_sign_message
        }

        verify_message: {
            inputs: |message: String, provenance: Provenance|,
            outputs: |result: ZomeApiResult<bool>|,
            handler: handle_verify_message
        }
    ]

    traits: {
        hc_public [sign_message, verify_message]
    }
}
