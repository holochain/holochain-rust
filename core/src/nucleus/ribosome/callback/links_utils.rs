use context::Context;
use futures::{executor::block_on, FutureExt};
use holochain_core_types::{
    dna::wasm::DnaWasm,
    entry::{
        Entry,
        entry_type::EntryType
    },
    error::HolochainError,
    link::Link,
};
use nucleus::actions::get_entry::get_entry;
use std::sync::Arc;
use holochain_wasm_utils::api_serialization::validation::{
    LinkDirection, LinkValidationPackageArgs,
};


pub fn get_link_entries(link: &Link, context: &Arc<Context>) -> Result<(Entry,Entry), HolochainError> {
    let base_address = link.base();
    let target_address = link.target();
    let base = block_on(get_entry(&context, base_address.clone()))?
        .ok_or(HolochainError::ErrorGeneric(String::from("Could not find link base")))?;
    let target = block_on(get_entry(&context, target_address.clone()))?
        .ok_or(HolochainError::ErrorGeneric(String::from("Could not find link target")))?;
    Ok((base, target))
}

pub struct LinkDefinitionPath {
    pub zome_name: String,
    pub entry_type_name: String,
    pub direction: LinkDirection,
    pub tag: String,
}

pub fn find_link_definition_in_dna(base_type: &EntryType, tag: &String, target_type: &EntryType, context: &Arc<Context>) -> Option<LinkDefinitionPath> {
    let dna = context.get_dna().expect("No DNA found?!");
    match base_type {
        EntryType::App(app_entry_type) => {
            dna.get_entry_type_def(&app_entry_type)
                .expect("Found entry type that is not defined in DNA?!")
                .links_to
                .iter()
                .find(|&link_def|
                    link_def.target_type == String::from(target_type.clone()) &&
                        &link_def.tag == tag
                )
                .and_then(|link_def| {
                    Some(
                        LinkDefinitionPath {
                            zome_name: dna.get_zome_name_for_entry_type(app_entry_type)
                                .expect("App entry types must be defined"),
                            entry_type_name: app_entry_type.clone(),
                            direction: LinkDirection::To,
                            tag: tag.clone(),
                        }
                    )
                })
        },
        _ => None
    }

        .or(
            match target_type {
                EntryType::App(app_entry_type) => {
                    dna.get_entry_type_def(&app_entry_type)
                        .expect("Found entry type that is not defined in DNA?!")
                        .linked_from
                        .iter()
                        .find(|&link_def|
                            link_def.base_type == String::from(base_type.clone()) &&
                                &link_def.tag == tag
                        )
                        .and_then(|_| {
                            Some(
                                LinkDefinitionPath {
                                    zome_name: dna.get_zome_name_for_entry_type(app_entry_type)
                                        .expect("App entry types must be defined"),
                                    entry_type_name: app_entry_type.clone(),
                                    direction: LinkDirection::From,
                                    tag: tag.clone(),
                                }
                            )
                        })
                },
                _ => None
            }
        )
}

