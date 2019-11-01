/// This file holds everything that represents the "post" entry type.
use hdk::prelude::*;
use boolinator::Boolinator;

/// We declare the structure of our entry type with this Rust struct.
/// It will be checked automatically by the macro below, similar
/// to how this happens with functions parameters and zome_functions!.
///
/// So this is our normative schema definition:
#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Post {
    pub content: String,
    pub date_created: String,
}

impl Post {
    pub fn new(content: &str, date_created: &str) -> Post {
        Post {
            content: content.to_owned(),
            date_created: date_created.to_owned(),
        }
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }

    pub fn date_created(&self) -> String {
        self.date_created.clone()
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

        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: |validation_data: hdk::EntryValidationData<Post>| {
            match validation_data
            {
                EntryValidationData::Create{entry:post,validation_data:_} =>
                {
                    (post.content.len() < 280)
                   .ok_or_else(|| String::from("Content too long"))
                },
                EntryValidationData::Modify{new_entry:new_post,old_entry:old_post,old_entry_header:_,validation_data:_} =>
                {
                   (new_post.content != old_post.content)
                   .ok_or_else(|| String::from("Trying to modify with same data"))
                },
                EntryValidationData::Delete{old_entry:old_post,old_entry_header:_,validation_data:_} =>
                {
                   (old_post.content!="SYS")
                   .ok_or_else(|| String::from("Trying to delete native type with content SYS"))
                }

            }

        },

        links: [
            from!(
                "%agent_id",
                link_type: "authored_posts",
                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },
                validation: | validation_data: hdk::LinkValidationData | {
                    // test validation of links based on their tag
                    if let hdk::LinkValidationData::LinkAdd{link, ..} = validation_data {
                        if link.link.tag() == "muffins" {
                            Err("This is the one tag that is not allowed!".into())
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
            ),
            from!(
                "%agent_id",
                link_type: "recommended_posts",
                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },
                validation: | _validation_data: hdk::LinkValidationData | {
                    Ok(())
                }
            )
        ]
    )
}

#[cfg(test)]
mod tests {

    use crate::post::{definition, Post};
    use hdk::{
        holochain_core_types::{
            chain_header::test_chain_header,
            dna::entry_types::{EntryTypeDef, LinkedFrom, Sharing},
            entry::{
                entry_type::{AppEntryType, EntryType},
                Entry,
            },
            validation::{EntryLifecycle, EntryValidationData, ValidationData, ValidationPackage},
        },
        holochain_json_api::json::JsonString,
        holochain_wasm_utils::api_serialization::validation::LinkDirection,
    };
    use std::convert::TryInto;

    #[test]
    /// smoke test Post
    fn post_smoke_test() {
        let content = "foo";
        let date_created = "bar";
        let post = Post::new(content, date_created);

        assert_eq!(content.to_string(), post.content(),);

        assert_eq!(date_created.to_string(), post.date_created(),);
    }

    #[test]
    fn post_definition_test() {
        let mut post_definition = definition();

        let expected_name = EntryType::from("post");
        assert_eq!(expected_name, post_definition.name.clone());

        let expected_definition = EntryTypeDef {
            properties: JsonString::from("blog entry post"),
            linked_from: vec![
                LinkedFrom {
                    base_type: "%agent_id".to_string(),
                    link_type: "authored_posts".to_string(),
                },
                LinkedFrom {
                    base_type: "%agent_id".to_string(),
                    link_type: "recommended_posts".to_string(),
                },
            ],
            links_to: Vec::new(),
            sharing: Sharing::Public,
        };
        assert_eq!(
            expected_definition,
            post_definition.entry_type_definition.clone(),
        );

        let expected_validation_package_definition = hdk::ValidationPackageDefinition::ChainFull;
        assert_eq!(
            expected_validation_package_definition,
            (post_definition.package_creator)(),
        );

        let post_ok = Post::new("foo", "now");
        let entry = Entry::App(AppEntryType::from("post"), post_ok.into());
        let validation_data = ValidationData {
            package: ValidationPackage::only_header(test_chain_header()),
            lifecycle: EntryLifecycle::Chain,
        };
        assert_eq!(
            (post_definition.validator)(EntryValidationData::Create {
                entry,
                validation_data
            }),
            Ok(()),
        );

        let post_not_ok = Post::new(
            "Tattooed organic sartorial, tumeric cray truffaut kale chips farm-to-table vaporware seitan brooklyn vegan locavore fam mixtape. Kale chips cold-pressed yuccie kickstarter yr. Fanny pack chambray migas heirloom microdosing blog, palo santo locavore cardigan swag organic. Disrupt pug roof party everyday carry kinfolk brooklyn quinoa. Flannel dreamcatcher yr blog, banjo hella brooklyn taxidermy four loko kickstarter aesthetic glossier biodiesel hot chicken heirloom. Leggings cronut helvetica yuccie meh.",
            "now",
        );

        let entry = Entry::App(
            post_definition.name.clone().try_into().unwrap(),
            post_not_ok.into(),
        );
        let validation_data = ValidationData {
            package: ValidationPackage::only_header(test_chain_header()),
            lifecycle: EntryLifecycle::Chain,
        };
        assert_eq!(
            (post_definition.validator)(EntryValidationData::Create {
                entry,
                validation_data
            }),
            Err("Content too long".to_string()),
        );

        let post_definition_link = post_definition.links.first().unwrap();

        let expected_link_base = "%agent_id";
        assert_eq!(
            post_definition_link.other_entry_type.to_owned(),
            expected_link_base,
        );

        let expected_link_direction = LinkDirection::From;
        assert_eq!(
            post_definition_link.direction.to_owned(),
            expected_link_direction,
        );

        let expected_link_tag = "authored_posts";
        assert_eq!(post_definition_link.link_type.to_owned(), expected_link_tag,);
    }
}
