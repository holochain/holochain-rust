#![feature(try_from)]

use hdk::{
    entry_definition::ValidatingEntryType,
    error::ZomeApiResult,
    holochain_core_types::{
        cas::content::Address, dna::entry_types::Sharing, entry::Entry, error::HolochainError,
        json::JsonString,
    },
};

pub fn handle_sign_me_message(entry: MyEntry) -> ZomeApiResult<Address> {
    let entry = Entry::App("my_entry".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

define_zome! {
    entries: []

    genesis: || { Ok(()) }

    functions: [
        sign_me_message: {
            inputs: |entry: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_sign_me_message
        }
    ]

    traits: {
        hc_public [create_my_entry]
    }
}
