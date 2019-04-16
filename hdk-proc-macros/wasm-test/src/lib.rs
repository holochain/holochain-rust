#![feature(try_from)]
#![feature(proc_macro_hygiene)]
extern crate hdk_proc_macros;
use hdk_proc_macros::zome;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate hdk;
#[macro_use]
extern crate holochain_core_types_derive;

use hdk::{
    error::ZomeApiResult,
    holochain_core_types::{
        dna::entry_types::Sharing,
        json::JsonString,
        error::{HolochainError},
    },
};

#[zome]
pub mod someZome {

    #[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
    struct TestEntryType {
        stuff: String,
    }
    
    #[genesis]
    fn genisis() {
        Ok(())
    }

    #[zome_fn("hc_public", "trait2")]
    fn test_zome_fn(input: i32, next: bool, another: JsonString) -> JsonString {
        JsonString::from_json("hi")
    }

    #[zome_fn("trait3")]
    fn test_zome_fn2(input: i32, next: bool, another: TestEntryType) -> ZomeApiResult<JsonString> {
        Ok(JsonString::from_json("hi"))
    }

    #[entry_def]
    fn test_entry_def() -> hdk::entry_definition::ValidatingEntryType {
        entry!(
            name: "testEntryType",
            description: "asdfda",
            sharing: Sharing::Public,
            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },
            validation: |_validation_data: hdk::EntryValidationData<TestEntryType>| {
                Ok(())
            }
        )
    }

    #[receive]
    fn glerp_glerp(message: String) -> String {
        message
    }
    
}
