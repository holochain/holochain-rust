#![feature(try_from)]
#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_core_types_derive;

use hdk::{
    entry_definition::ValidatingEntryType,
    error::ZomeApiResult,
};
use hdk::holochain_core_types::{
    cas::content::Address,
    dna::entry_types::Sharing,
    error::HolochainError,
    json::JsonString
};

use hdk::holochain_wasm_utils::api_serialization::get_links::GetLinksResult;


// see https://developer.holochain.org/api/0.0.18-alpha1/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct MyEntry {
    content: String,
}



pub fn handle_create_my_link(base: Address,target : Address) -> ZomeApiResult<()> {
    hdk::link_entries(&base, &target, "authored_posts", "")?;
    Ok(())
}

pub fn handle_delete_my_link(base: Address,target : Address) -> ZomeApiResult<()> {
    hdk::remove_link(&base, &target, "authored_posts", "")?;
    Ok(())
}

pub fn handle_get_my_links(base: Address) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&base, Some("authored_posts".into()), None)
}

fn definition() -> ValidatingEntryType {
    entry!(
        name: "simple",
        description: "this is a simple definition for lightweight app_spec tests",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },

        validation: | _validation_data: hdk::EntryValidationData<MyEntry>| {
            Ok(())
        }
    )
}

define_zome! {
    entries: [
       definition()
    ]

    genesis: || { Ok(()) }

    functions: [
        create_link: {
            inputs: |base : Address,target:Address|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_create_my_link
        }
        delete_link: {
            inputs: |base : Address,target:Address|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_delete_my_link
        }
        get_my_links: {
            inputs: |base: Address|,
            outputs: |result: ZomeApiResult<GetLinksResult>|,
            handler: handle_get_my_links
        }
    ]

    traits: {
        hc_public [create_link,delete_link,get_my_links]
    }
}
