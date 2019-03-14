#![feature(try_from)]

#[macro_use]
extern crate hdk;
#[macro_use]
extern crate serde_derive;

use hdk::{
    error::ZomeApiResult,
    holochain_core_types::{
        cas::content::Address, entry::Entry, error::HolochainError, json::JsonString,
    },
};

pub fn handle_sign_me_message(message: String) -> ZomeApiResult<String> {
    Ok("address".into())
}

pub fn handle_verify_me_message(entry: String) -> ZomeApiResult<bool> {
    Ok(false)
}

define_zome! {
    entries: []

    genesis: || { Ok(()) }

    functions: [
        sign_me_message: {
            inputs: |message: String|,
            outputs: |result: ZomeApiResult<String>|,
            handler: handle_sign_me_message
        }

        verify_me_message: {
            inputs: |entry: String|,
            outputs: |result: ZomeApiResult<bool>|,
            handler: handle_verify_me_message
        }
    ]

    traits: {
        hc_public [sign_me_message, verify_me_message]
    }
}
