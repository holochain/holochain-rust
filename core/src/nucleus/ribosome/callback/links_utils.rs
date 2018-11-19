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


fn wasm_for_entry_type(entry_type: &String, context: &Arc<Context>) -> DnaWasm {
    let dna = context.get_dna().expect("No DNA found?!");
    let zome_name = dna.get_zome_name_for_entry_type(entry_type)
        .expect("App entry types must be defined");
    context.get_wasm(&zome_name)
        .expect("Zomes must have a WASM binary")
}

pub fn find_link_definition_in_dna(base_type: &EntryType, tag: &String, target_type: &EntryType, context: &Arc<Context>) -> Option<(LinkValidationPackageArgs, DnaWasm)> {
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
                    Some((
                        LinkValidationPackageArgs {
                            entry_type: app_entry_type.clone(),
                            tag: tag.clone(),
                            direction: LinkDirection::To,
                        },
                        wasm_for_entry_type(&app_entry_type, context)
                    ))
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
                            Some((
                                LinkValidationPackageArgs {
                                    entry_type: app_entry_type.clone(),
                                    tag: tag.clone(),
                                    direction: LinkDirection::From,
                                },
                                wasm_for_entry_type(&app_entry_type, context)
                            ))
                        })
                },
                _ => None
            }
        )
}

