#![feature(try_from)]
#[macro_use]
extern crate hdk;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_core_types_derive;


use hdk::holochain_core_types::{
    dna::entry_types::Sharing,
    error::HolochainError,
    json::{JsonString},
};

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

            validation: | validation_data: hdk::ValidationData| {
                /*(String::from(s) != String::from("FAIL"))
                    .ok_or_else(|| "FAIL content is not allowed".to_string())*/
                    Ok(())
            }
        ),

        entry!(
            name: "package_entry",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },

            validation: |validation_data: hdk::ValidationData| {
                /*(entry.stuff != "FAIL")
                    .ok_or_else(|| "FAIL content is not allowed".to_string())*/
                Ok(())
            }
        ),

        entry!(
            name: "package_chain_entries",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainEntries
            },

            validation: |_validation_data: hdk::ValidationData| {
                /*(entry.stuff != "FAIL")
                    .ok_or_else(|| "FAIL content is not allowed".to_string())*/
                    Ok(())
            }
        ),

        entry!(
            name: "package_chain_headers",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainHeaders
            },

            validation: |_validation_data: hdk::ValidationData| {
                /*(entry.stuff != "FAIL")
                    .ok_or_else(|| "FAIL content is not allowed".to_string())*/
                Ok(())
            }
        ),

        entry!(
            name: "package_chain_full",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },

            validation: | _validation_data: hdk::ValidationData| {
                /*(entry.stuff != "FAIL")
                    .ok_or_else(|| "FAIL content is not allowed".to_string())*/
                    Ok(())
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

fn test_handler() -> u32 {
    0
}
