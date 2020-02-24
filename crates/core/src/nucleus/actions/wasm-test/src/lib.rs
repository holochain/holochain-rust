extern crate hdk;

extern crate boolinator;
extern crate holochain_wasmer_guest;

use holochain_wasmer_guest::*;
use hdk::holochain_core_types::{
    dna::entry_types::Sharing,
    validation::EntryValidationData
};

use hdk::prelude::*;

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
                   EntryValidationData::Create{entry:test_entry,validation_data:_} =>
                   {
                        if test_entry.stuff != "FAIL" {
                            ValidationResult::Ok
                         }
                         else {
                             ValidationResult::Err(ValidationError::Fail("FAIL content is not allowed".into()))
                         }
                   }
                   _ =>{
                       ValidationResult::Err(ValidationError::Fail("Failed to validate with wrong entry type".into()))
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
                   EntryValidationData::Create{entry:test_entry,validation_data:_} =>
                   {
                        if test_entry.stuff != "FAIL" {
                            ValidationResult::Ok
                        } else {
                         ValidationResult::Err(ValidationError::Fail("FAIL content is not allowed".into()))
                     }
                   }
                   _ =>{
                       ValidationResult::Err(ValidationError::Fail("Failed to validate with wrong entry type".into()))
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
                   EntryValidationData::Create{entry:test_entry,validation_data:_} =>
                   {
                        if test_entry.stuff != "FAIL" {
                            ValidationResult::Ok
                        } else {
                            ValidationResult::Err(ValidationError::Fail("FAIL content is not allowed".into()))
                        }
                   }
                   _ =>{
                       ValidationResult::Err(ValidationError::Fail("Failed to validate with wrong entry type".into()))
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
                   EntryValidationData::Create{entry:test_entry,validation_data:_} =>
                   {

                        if test_entry.stuff != "FAIL" {
                            ValidationResult::Ok
                        } else {
                            ValidationResult::Err(ValidationError::Fail("FAIL content is not allowed".into()))
                        }
                   }
                   _ =>{
                       ValidationResult::Err(ValidationError::Fail("Failed to validate with wrong entry type".into()))
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
                   EntryValidationData::Create{entry:test_entry,validation_data:_} =>
                   {

                        if test_entry.stuff != "FAIL" {
                            ValidationResult::Ok
                        } else {
                            ValidationResult::Err(ValidationError::Fail("FAIL content is not allowed".into()))
                        }
                   }
                   _ =>{
                       ValidationResult::Err(ValidationError::Fail("Failed to validate with wrong entry type".into()))
                   }
                }
            }
        )
    ]

    init: || {
        Ok(())
    }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        ValidationResult::Ok
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
