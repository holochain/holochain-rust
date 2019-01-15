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
    cas::content::Address, entry::Entry, dna::entry_types::Sharing, error::HolochainError, json::JsonString,
};

// see https://developer.holochain.org/api/0.0.3/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
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
        native_type: MyEntry,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },

        validation: |_my_entry: MyEntry, _ctx: hdk::ValidationData| {
            Ok(())
        }
    )
}
define_zome! {
    entries: [
       definition()
    ]

    genesis: || { Ok(()) }

    functions: {
        main (Public) {
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
        }
    }
}
