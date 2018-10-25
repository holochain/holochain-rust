#[macro_use]
extern crate hdk;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;

use boolinator::Boolinator;
use hdk::holochain_dna::zome::entry_types::Sharing;

#[derive(Serialize, Deserialize)]
struct TestEntryType {
    stuff: String,
}


define_zome! {
    entries: [
        entry!(
            name: "package_entry",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },

            validation: |entry: TestEntryType, _ctx: hdk::ValidationData| {
                (entry.stuff != "FAIL")
                    .ok_or_else(|| "FAIL content is not allowed".to_string())
            }
        ),

        entry!(
            name: "package_chain_entries",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainEntries
            },

            validation: |entry: TestEntryType, _ctx: hdk::ValidationData| {
                (entry.stuff != "FAIL")
                    .ok_or_else(|| "FAIL content is not allowed".to_string())
            }
        ),

        entry!(
            name: "package_chain_headers",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainHeaders
            },

            validation: |entry: TestEntryType, _ctx: hdk::ValidationData| {
                (entry.stuff != "FAIL")
                    .ok_or_else(|| "FAIL content is not allowed".to_string())
            }
        ),

        entry!(
            name: "package_chain_full",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },

            validation: |entry: TestEntryType, _ctx: hdk::ValidationData| {
                (entry.stuff != "FAIL")
                    .ok_or_else(|| "FAIL content is not allowed".to_string())
            }
        )
    ]

    genesis: || {
        Ok(())
    }

    functions: {

    }
}
