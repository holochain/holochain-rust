//use boolinator::Boolinator;
use hdk::entry_definition::ValidatingEntryType;
/// This file holds everything that represents the "post" entry type.
use hdk::holochain_core_types::{
    dna::entry_types::Sharing, error::HolochainError, json::JsonString,
};

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub enum QueryType {
    And,
    Or
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Topic {
    pub topic: String
}

impl Topic {
    pub fn new(topic: &str) -> Topic {
        Topic {
            topic: topic.to_owned()
        }
    }

    pub fn content(&self) -> String {
        self.topic.clone()
    }
}

pub fn definition() -> ValidatingEntryType {
    entry!(
        name: "topic",
        description: "A topic entry - used for indexing posts based on their topic",
        sharing: Sharing::Public,

        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: |_validation_data: hdk::EntryValidationData<Topic>| {
            Ok(())
        },

        links: [
            to!(
                "post",
                link_type: "topic_index",
                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },
                validation: |_validation_data: hdk::LinkValidationData| {
                    Ok(())
                }   
            )
        ]
    )
}

#[cfg(test)]
mod tests {
    use crate::topic::{
        Topic,
        definition
    };
    use hdk::{
        holochain_core_types::{
            chain_header::test_chain_header,
            dna::entry_types::{EntryTypeDef, LinksTo, Sharing},
            entry::{
                entry_type::{AppEntryType, EntryType},
                Entry,
            },
            validation::{EntryLifecycle, EntryValidationData, ValidationData, ValidationPackage},
        },
        holochain_wasm_utils::api_serialization::validation::LinkDirection,
    };

    #[test]
    fn time_smoke_test() {
        let content = "test-topic";
        let topic = Topic::new(content);

        assert_eq!(content.to_string(), topic.content(),);
    }

    #[test]
    fn time_definition_test() {
        let mut topic_definition = definition();

        let expected_name = EntryType::from("topic");
        assert_eq!(expected_name, topic_definition.name.clone());

        let expected_definition = EntryTypeDef {
            description: "A topic entry - used for indexing posts based on their topic".to_string(),
            linked_from: Vec::new(),
            links_to: vec![          
                LinksTo {
                    target_type: "post".to_string(),
                    link_type: "topic_index".to_string(),
                }
            ],
            sharing: Sharing::Public,
        };
        assert_eq!(
            expected_definition,
            topic_definition.entry_type_definition.clone(),
        );

        let expected_validation_package_definition = hdk::ValidationPackageDefinition::ChainFull;
        assert_eq!(
            expected_validation_package_definition,
            (topic_definition.package_creator)(),
        );

        let post_ok = Topic::new("foo");
        let entry = Entry::App(AppEntryType::from("topic"), post_ok.into());
        let validation_data = ValidationData {
            package: ValidationPackage::only_header(test_chain_header()),
            lifecycle: EntryLifecycle::Chain,
        };
        assert_eq!(
            (topic_definition.validator)(EntryValidationData::Create {
                entry,
                validation_data
            }),
            Ok(()),
        );

        let topic_definition_link = topic_definition.links.first().unwrap();

        let expected_link_base = "post";
        assert_eq!(
            topic_definition_link.other_entry_type.to_owned(),
            expected_link_base,
        );

        let expected_link_direction = LinkDirection::To;
        assert_eq!(
            topic_definition_link.direction.to_owned(),
            expected_link_direction,
        );

        let expected_link_type = "topic_index";
        assert_eq!(topic_definition_link.link_type.to_owned(), expected_link_type,);
    }
}
