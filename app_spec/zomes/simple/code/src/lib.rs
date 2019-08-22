#![warn(unused_extern_crates)]
#[macro_use]
extern crate hdk;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_json_derive;

use hdk::{
    entry_definition::ValidatingEntryType,
    error::ZomeApiResult,
};
use hdk::holochain_core_types::{
    dna::entry_types::Sharing,
    entry::Entry,
};
use hdk::holochain_persistence_api::{
    cas::content::Address,
};
use hdk::holochain_json_api::{
    json::JsonString,
    error::JsonError
};


// see https://developer.holochain.org/api/latest/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct Simple {
    content: String,
}

impl Simple {
    pub fn new(content:String) -> Simple {
        Simple{content}
    }
}

fn simple_entry(content: String) -> Entry {
    Entry::App("simple".into(), Simple::new(content).into())
}

pub fn definition() -> ValidatingEntryType {
    entry!(
        name: "simple",
        description: "this is a simple definition for lightweight app_spec tests",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },

        validation: | _validation_data: hdk::EntryValidationData<Simple>| {
            Ok(())
        }
    )
}

fn commit_entry_handler(content: String) -> ZomeApiResult<Address> {
    hdk::commit_entry(&simple_entry(content))
}

define_zome! {

    entries: [
       definition()
    ]

    init: || {
        Ok(())
    }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {{
        Ok(())
    }}

    functions: [
        commit_entry: {
            inputs: |content: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: commit_entry_handler
        }
    ]

    traits: {
        hc_public [commit_entry]
    }
}
