#![feature(try_from)]
#[macro_use]
extern crate hdk;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_core_types_derive;

extern crate boolinator;

use boolinator::Boolinator;


use hdk::holochain_core_types::{
    dna::entry_types::Sharing,
    error::HolochainError,
    json::{JsonString},
    validation::EntryValidationData
};



 


#[derive(Serialize, Deserialize, DefaultJson, Debug,Clone)]
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

            validation: | validation_data: hdk::EntryValidationData<TestEntryType>| {
                 match validation_data
                 {
                   EntryValidationData::Create(test_entry) =>
                   {
                        (test_entry.stuff != "FAIL")
                        .ok_or_else(|| "FAIL content is not allowed".to_string())
                   }
                   _ =>{
                       Err("Failed to validate with wrong entry type".to_string())
                   }
                }
            }
        ),

        entry!(
            name: "package_entry",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },

            validation: |validation_data: hdk::EntryValidationData<TestEntryType>| {
                match validation_data
                {
                   EntryValidationData::Create(test_entry) =>
                   {
                        
                        (test_entry.stuff != "FAIL")
                        .ok_or_else(|| "FAIL content is not allowed".to_string())
                   }
                   _ =>{
                       Err("Failed to validate with wrong entry type".to_string())
                   }
                }
               
            }
        ),

        entry!(
            name: "package_chain_entries",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainEntries
            },

            validation: |validation_data: hdk::EntryValidationData<TestEntryType>| {
                 match validation_data
                {
                   EntryValidationData::Create(test_entry) =>
                   {
                        
                        (test_entry.stuff != "FAIL")
                        .ok_or_else(|| "FAIL content is not allowed".to_string())
                   }
                   _ =>{
                       Err("Failed to validate with wrong entry type".to_string())
                   }
                }
            }
        ),

        entry!(
            name: "package_chain_headers",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainHeaders
            },

            validation: |validation_data: hdk::EntryValidationData<TestEntryType>| {
                 match validation_data
                {
                   EntryValidationData::Create(test_entry) =>
                   {
                        
                        (test_entry.stuff != "FAIL")
                        .ok_or_else(|| "FAIL content is not allowed".to_string())
                   }
                   _ =>{
                       Err("Failed to validate with wrong entry type".to_string())
                   }
                }
            }
        ),

        entry!(
            name: "package_chain_full",
            description: "asdfda",
            sharing: Sharing::Public,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },

            validation: | validation_data: hdk::EntryValidationData<TestEntryType>| {
                 match validation_data
                {
                   EntryValidationData::Create(test_entry) =>
                   {
                        
                        (test_entry.stuff != "FAIL")
                        .ok_or_else(|| "FAIL content is not allowed".to_string())
                   }
                   _ =>{
                       Err("Failed to validate with wrong entry type".to_string())
                   }
                }
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
