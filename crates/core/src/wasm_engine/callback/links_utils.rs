use crate::{context::Context, workflows::get_entry_result::get_entry_result_workflow};
use holochain_core_types::{
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    link::Link,
};
use holochain_wasm_utils::api_serialization::{get_entry::*, validation::LinkDirection};
use std::sync::Arc;

/// Retrieves the base and target entries of the link and returns both.
#[autotrace]
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn get_link_entries(
    link: &Link,
    context: &Arc<Context>,
) -> Result<(Entry, Entry), HolochainError> {
    let base_address = link.base();
    let target_address = link.target();
    let entry_args = &GetEntryArgs {
        address: base_address.clone(),
        options: Default::default(),
    };
    let base_entry_get_result =
        context.block_on(get_entry_result_workflow(&context, entry_args))?;
    if !base_entry_get_result.found() {
        return Err(HolochainError::ErrorGeneric(String::from(
            "Base for link not found",
        )));
    }
    let base_entry = base_entry_get_result.latest().unwrap();
    let entry_args = &GetEntryArgs {
        address: target_address.clone(),
        options: Default::default(),
    };
    let target_entry_get_result =
        context.block_on(get_entry_result_workflow(&context, entry_args))?;
    if !target_entry_get_result.found() {
        return Err(HolochainError::ErrorGeneric(String::from(
            "Target for link not found",
        )));
    }

    Ok((base_entry, target_entry_get_result.latest().unwrap()))
}

/// This is a "path" in the DNA tree.
/// That uniquely identifies a link definition.
///
/// zome
///  |_ entry type
///      |_ direction (links_to / linked_from)
///          |_ link_type
///
/// Needed for link validation to call the right callback
pub struct LinkDefinitionPath {
    pub zome_name: String,
    pub entry_type_name: String,
    pub direction: LinkDirection,
    pub link_type: String,
}

/// This function tries to find the link definition for a link given by link type.
/// It assumes that link type names are unique and thus just iterates through
/// zomes, entry types and their links and returns the first match.
///
/// Returns a LinkDefinitionPath to uniquely reference the link definition in the DNA.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn find_link_definition_by_type(
    link_type: &String,
    context: &Arc<Context>,
) -> Result<LinkDefinitionPath, HolochainError> {
    let dna = context.get_dna().expect("No DNA found?!");
    for (zome_name, zome) in dna.zomes.iter() {
        for (entry_type, entry_type_def) in zome.entry_types.iter() {
            if let EntryType::App(entry_type_name) = entry_type.clone() {
                for link in entry_type_def.links_to.iter() {
                    if link.link_type == *link_type {
                        return Ok(LinkDefinitionPath {
                            zome_name: zome_name.clone(),
                            entry_type_name: entry_type_name.to_string(),
                            direction: LinkDirection::To,
                            link_type: link_type.clone(),
                        });
                    }
                }

                for link in entry_type_def.linked_from.iter() {
                    if link.link_type == *link_type {
                        return Ok(LinkDefinitionPath {
                            zome_name: zome_name.clone(),
                            entry_type_name: entry_type_name.to_string(),
                            direction: LinkDirection::From,
                            link_type: link_type.clone(),
                        });
                    }
                }
            }
        }
    }

    Err(HolochainError::ErrorGeneric(String::from(
        "Unknown entry type",
    )))
}
