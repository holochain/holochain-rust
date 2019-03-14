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

pub fn handle_sign_me_message(message: String) -> ZomeApiResult<Address> {
    let entry = Entry::App("my_entry".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_verify_me_message(entry: String) -> ZomeApiResult<Address> {
    let entry = Entry::App("my_entry".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

define_zome! {
    entries: []

    genesis: || { Ok(()) }

    functions: [
        sign_me_message: {
            inputs: |message: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_sign_me_message
        }

        verify_me_message: {
            inputs: |entry: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_verify_me_message
        }
    ]

    traits: {
        hc_public [sign_me_message, verify_me_message]
    }
}
