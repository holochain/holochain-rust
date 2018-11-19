use context::Context;
use futures::executor::block_on;
use holochain_core_types::{
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    link::Link,
};
use holochain_wasm_utils::api_serialization::validation::LinkDirection;
use nucleus::actions::get_entry::get_entry;
use std::sync::Arc;

pub fn get_link_entries(
    link: &Link,
    context: &Arc<Context>,
) -> Result<(Entry, Entry), HolochainError> {
    let base_address = link.base();
    let target_address = link.target();
    let base = block_on(get_entry(&context, base_address.clone()))?.ok_or(
        HolochainError::ErrorGeneric(String::from("Base for link not found")),
    )?;
    let target = block_on(get_entry(&context, target_address.clone()))?.ok_or(
        HolochainError::ErrorGeneric(String::from("Target for link not found")),
    )?;
    Ok((base, target))
}

pub struct LinkDefinitionPath {
    pub zome_name: String,
    pub entry_type_name: String,
    pub direction: LinkDirection,
    pub tag: String,
}

pub fn find_link_definition_in_dna(
    base_type: &EntryType,
    tag: &String,
    target_type: &EntryType,
    context: &Arc<Context>,
) -> Result<LinkDefinitionPath, HolochainError> {
    let dna = context.get_dna().expect("No DNA found?!");
    match base_type {
        EntryType::App(app_entry_type) => dna
            .get_entry_type_def(&app_entry_type)
            .ok_or(HolochainError::ErrorGeneric(String::from("Unknown entry type")))?
            .links_to
            .iter()
            .find(|&link_def| {
                link_def.target_type == String::from(target_type.clone()) && &link_def.tag == tag
            })
            .and_then(|link_def| {
                Some(LinkDefinitionPath {
                    zome_name: dna
                        .get_zome_name_for_entry_type(app_entry_type)?,
                    entry_type_name: app_entry_type.clone(),
                    direction: LinkDirection::To,
                    tag: link_def.tag.clone(),
                })
            }),
        _ => None,
    }.or(match target_type {
        EntryType::App(app_entry_type) => dna
            .get_entry_type_def(&app_entry_type)
            .ok_or(HolochainError::ErrorGeneric(String::from("Unknown entry type")))?
            .linked_from
            .iter()
            .find(|&link_def| {
                link_def.base_type == String::from(base_type.clone()) && &link_def.tag == tag
            })
            .and_then(|link_def| {
                Some(LinkDefinitionPath {
                    zome_name: dna
                        .get_zome_name_for_entry_type(app_entry_type)?,
                    entry_type_name: app_entry_type.clone(),
                    direction: LinkDirection::From,
                    tag: link_def.tag.clone(),
                })
            }),
        _ => None,
    })
        .ok_or(HolochainError::ErrorGeneric(String::from("Unknown entry type")))
}
