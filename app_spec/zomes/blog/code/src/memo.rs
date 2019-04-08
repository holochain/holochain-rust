/// This file holds everything that represents the "memo" entry type.
/// a Memo is essentially a private post that should never be publically
/// published on the dht.
use boolinator::Boolinator;
use hdk::entry_definition::ValidatingEntryType;
use hdk::holochain_core_types::{
     error::HolochainError, json::JsonString,
    validation::{EntryValidationData},
    dna::entry_types::Sharing
};

/// We declare the structure of our entry type with this Rust struct.
/// It will be checked automatically by the macro below, similar
/// to how this happens with functions parameters and zome_functions!.
///
/// So this is our normative schema definition:
#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct Memo {
    pub content: String,
    pub date_created: String,
}

impl Memo {
    pub fn new(content: &str, date_created: &str) -> Memo {
        Memo {
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
        name: "memo",
        description: "A private memo entry type.",
        sharing: Sharing::Private,

        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: |validation_data: hdk::EntryValidationData<Memo>| {
            match validation_data
            {
                EntryValidationData::Create{entry:memo,validation_data:_} => 
                {
                    (memo.content.len() < 280)
                   .ok_or_else(|| String::from("Content too long"))
                },
                EntryValidationData::Modify{new_entry:new_memo,old_entry:old_memo,old_entry_header:_,validation_data:_} =>
                {
                   (new_memo.content != old_memo.content)
                   .ok_or_else(|| String::from("Trying to modify with same data"))   
                },
                EntryValidationData::Delete{old_entry:old_memo,old_entry_header:_,validation_data:_} =>
                {
                   (old_memo.content!="SYS")
                   .ok_or_else(|| String::from("Trying to delete native type with content SYS"))   
                }

            }

        },

        links: [
        ]
    )
}

#[cfg(test)]
mod tests {

    use crate::memo::{definition, Memo};
    use hdk::{
        holochain_core_types::{
            dna::entry_types::{EntryTypeDef, LinkedFrom},
            entry::{entry_type::{EntryType,AppEntryType},Entry},
             dna::entry_types::Sharing,
             validation::{EntryValidationData,ValidationPackage,EntryLifecycle,ValidationData},
             chain_header::test_chain_header
        },
        holochain_wasm_utils::api_serialization::validation::LinkDirection
    };
    use std::convert::TryInto;

    #[test]
    /// smoke test Memo
    fn memo_smoke_test() {
        let content = "foo";
        let date_created = "bar";
        let memo = Memo::new(content, date_created);

        assert_eq!(content.to_string(), memo.content(),);

        assert_eq!(date_created.to_string(), memo.date_created(),);
    }

    #[test]
    fn memo_definition_test() {
        let mut memo_definition = definition();

        let expected_name = EntryType::from("memo");
        assert_eq!(expected_name, memo_definition.name.clone());

        let expected_definition = EntryTypeDef {
            description: "blog entry memo".to_string(),
            linked_from: vec![LinkedFrom {
                base_type: "%agent_id".to_string(),
                tag: "authored_memos".to_string(),
            },
            LinkedFrom {
                base_type: "%agent_id".to_string(),
                tag: "recommended_memos".to_string(),
            }],
            links_to : Vec::new(),
            sharing : Sharing::Public
        };
        assert_eq!(
            expected_definition,
            memo_definition.entry_type_definition.clone(),
        );

        let expected_validation_package_definition = hdk::ValidationPackageDefinition::ChainFull;
        assert_eq!(
            expected_validation_package_definition,
            (memo_definition.package_creator)(),
        );

        let memo_ok = Memo::new("foo", "now");
        let entry = Entry::App(AppEntryType::from("memo"),memo_ok.into());
        let validation_data = ValidationData{
            package : ValidationPackage::only_header(test_chain_header()),
            lifecycle : EntryLifecycle::Chain
        };
        assert_eq!(
            (memo_definition.validator)(
               EntryValidationData::Create{entry,validation_data}
            ),
            Ok(()),
        );

        let memo_not_ok = Memo::new(
            "Tattooed organic sartorial, tumeric cray truffaut kale chips farm-to-table vaporware seitan brooklyn vegan locavore fam mixtape. Kale chips cold-pressed yuccie kickstarter yr. Fanny pack chambray migas heirloom microdosing blog, palo santo locavore cardigan swag organic. Disrupt pug roof party everyday carry kinfolk brooklyn quinoa. Flannel dreamcatcher yr blog, banjo hella brooklyn taxidermy four loko kickstarter aesthetic glossier biodiesel hot chicken heirloom. Leggings cronut helvetica yuccie meh.",
            "now",
        );

        let entry = Entry::App(
                    memo_definition.name.clone().try_into().unwrap(),
                    memo_not_ok.into(),
                );
        let validation_data = ValidationData{
            package : ValidationPackage::only_header(test_chain_header()),
            lifecycle : EntryLifecycle::Chain
        };
        assert_eq!(
            (memo_definition.validator)(
               EntryValidationData::Create{entry,validation_data}
            ),
            Err("Content too long".to_string()),
        );

        let memo_definition_link = memo_definition.links.first().unwrap();

        let expected_link_base = "%agent_id";
        assert_eq!(
            memo_definition_link.other_entry_type.to_owned(),
            expected_link_base,
        );

        let expected_link_direction = LinkDirection::From;
        assert_eq!(
            memo_definition_link.link_type.to_owned(),
            expected_link_direction,
        );

        let expected_link_tag = "authored_memos";
        assert_eq!(memo_definition_link.tag.to_owned(), expected_link_tag,);
    }
}
