/// This file holds everything that represents the "post" entry type.

use hdk::holochain_core_types::{
    error::HolochainError,
    dna::zome::entry_types::Sharing,
    json::JsonString,
    cas::content::Address,
};
use hdk::{
    entry_definition::ValidatingEntryType,
};
use boolinator::Boolinator;

/// We declare the structure of our entry type with this Rust struct.
/// It will be checked automatically by the macro below, similar
/// to how this happens with functions parameters and zome_functions!.
///
/// So this is our normative schema definition:
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct Post {
    content: String,
    date_created: String,
}

impl Post {
    pub fn new (content: &str, date_created: &str) -> Post {
        Post {
            content: content.to_owned(),
            date_created: date_created.to_owned(),
        }
    }
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
        description: "blog entry post",
        sharing: Sharing::Public,
        native_type: Post,

        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: |post: crate::post::Post, _ctx: hdk::ValidationData| {
            (post.content.len() < 280)
                .ok_or_else(|| String::from("Content too long"))
        },

        links: [
            from!(
                "%agent_id",
                tag: "authored_posts",
                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },
                validation: |_source: Address, _target: Address, _ctx: hdk::ValidationData | {
                    Ok(())
                }
            )
        ]
    )
}
