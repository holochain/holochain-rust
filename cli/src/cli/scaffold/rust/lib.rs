#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_json_derive;

use hdk::{
    entry_definition::ValidatingEntryType,
    error::ZomeApiResult,
};
use hdk::holochain_core_types::{
    entry::Entry,
    dna::entry_types::Sharing,
};

use hdk::holochain_persistence_api::{
    cas::content::Address,
};

use hdk::holochain_json_api::{
    error::JsonError,
    json::JsonString,
};


// see https://developer.holochain.org/api/0.0.25-alpha1/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct MyEntry {
    content: String,
}

pub fn handle_create_my_entry(entry: MyEntry) -> ZomeApiResult<Address> {
    let entry = Entry::App("my_entry".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_get_my_entry(address: Address) -> ZomeApiResult<Option<Entry>> {
    hdk::get_entry(&address)
}

fn definition() -> ValidatingEntryType {
    entry!(
        name: "my_entry",
        description: "this is a same entry defintion",
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

    init: || { Ok(()) }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        Ok(())
    }

    functions: [
        create_my_entry: {
            inputs: |entry: MyEntry|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_create_my_entry
        }
        get_my_entry: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<Option<Entry>>|,
            handler: handle_get_my_entry
        }
    ]

    traits: {
        hc_public [create_my_entry,get_my_entry]
    }
}
