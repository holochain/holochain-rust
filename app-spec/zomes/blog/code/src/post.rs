/// This file holds everything that represents the "post" entry type.

use hdk::holochain_core_types::{
    dna::zome::entry_types::Sharing,
    error::HolochainError,
    json::JsonString,
};
use boolinator::*;
use hdk::{
    self,
    entry_definition::ValidatingEntryType,
};
use serde_json;

/// We declare the structure of our entry type with this Rust struct.
/// It will be checked automatically by the macro below, similar
/// to how this happens with functions parameters and zome_functions!.
///
/// So this is our normative schema definition:
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct Post {
    pub content: String,
    pub date_created: String,
}


/// This is what creates the full definition of our entry type.
/// The entry! macro is wrapped in a function so that we can have the content
/// in this file but call it from zome_setup() in lib.rs, which is like the
/// zome's main().
///
/// We will soon be able to also replace the json files that currently hold
/// most of these values. The only field that is really used is the
/// validation_package callback.
/// The validation_function still has to be defined with the macro below.
pub fn definition() -> ValidatingEntryType {
    entry!(
        name: "post",
        description: "",
        sharing: Sharing::Public,
        native_type: Post,

        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: |post: Post, _ctx: hdk::ValidationData| {
            (post.content.len() < 280)
                .ok_or_else(|| String::from("Content too long"))
        }
    )
}
