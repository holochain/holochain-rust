#![feature(try_from)]
#[macro_use]
extern crate hdk;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;
#[macro_use]
extern crate holochain_core_types_derive;

use boolinator::Boolinator;
use hdk::holochain_core_types::dna::entry_types::Sharing;
use hdk::holochain_core_types::json::JsonString;
use hdk::holochain_core_types::json::RawString;
use hdk::holochain_core_types::error::HolochainError;

#[derive(Serialize, Deserialize, DefaultJson, Debug)]
struct TestEntryType {
    stuff: String,
}

define_zome! {
    entries: [
        entry!(
            name: "testEntryType",
            description: "asdfdaz",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },

            validation: |s: RawString, _ctx: hdk::ValidationData| {
                (String::from(s) != String::from("FAIL"))
                    .ok_or_else(|| "FAIL content is not allowed".to_string())
            }
        ),

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

    functions: [
        test_fn: {
            inputs: | |,
            outputs: | x:u32 |,
            handler: test_handler
        }
    ]

    traits: {}
}

fn test_handler() -> u32 {0}
