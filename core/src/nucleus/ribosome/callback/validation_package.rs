extern crate serde_json;
use context::Context;
use futures::{executor::block_on, FutureExt};
use holochain_core_types::{
    dna::{
        Dna,
        wasm::DnaWasm
    },
    entry::{
        Entry, ToEntry,
        entry_type::EntryType
    },
    error::HolochainError, validation::ValidationPackageDefinition,
    link::link_add::LinkAddEntry,
};
use nucleus::{
    actions::get_entry::get_entry,
    ribosome::{
        self,
        callback::{get_dna, get_wasm, CallbackResult},
    },
    ZomeFnCall,
};
use std::{convert::TryFrom, sync::Arc};
use holochain_wasm_utils::api_serialization::validation::{
    LinkDirection, LinkValidationPackageArgs,
};

pub fn get_validation_package_definition(
    entry: &Entry,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    let dna = get_dna(&context).expect("Callback called without DNA set!");
    let result = match entry.entry_type().clone() {
        EntryType::App(app_entry_type) => {
            let zome_name = dna.get_zome_name_for_entry_type(&app_entry_type);
            if zome_name.is_none() {
                return Ok(CallbackResult::NotImplemented);
            }

            let zome_name = zome_name.unwrap();
            let wasm = get_wasm(&context, &zome_name)
                .ok_or(HolochainError::ErrorGeneric(String::from("no wasm found")))?;

            ribosome::run_dna(
                &dna.name.clone(),
                context,
                wasm.code.clone(),
                &ZomeFnCall::new(
                    &zome_name,
                    "no capability, since this is an entry validation call",
                    "__hdk_get_validation_package_for_entry_type",
                    app_entry_type.clone(),
                ),
                Some(app_entry_type.into_bytes()),
            )?
        },
        EntryType::LinkAdd => {
            let link_add_entry = LinkAddEntry::from_entry(entry);
            let link = link_add_entry.link();
            let base_address = link_add_entry.link().base();
            let target_address = link_add_entry.link().target();
            let base = block_on(get_entry(&context, base_address.clone()))?
                .ok_or(HolochainError::ErrorGeneric(String::from("Could not find link base")))?;
            let target = block_on(get_entry(&context, target_address.clone()))?
                .ok_or(HolochainError::ErrorGeneric(String::from("Could not find link target")))?;

            let (args, wasm) = find_link_definition_in_dna(
                &base.entry_type(),
                link.tag(),
                &target.entry_type(),
                &dna,
                &context,
            ).ok_or(HolochainError::NotImplemented)?;

            let call = ZomeFnCall::new(
                "",
                "no capability, since this is an entry validation call",
                "__hdk_get_validation_package_for_link",
                args.clone(),
            );

            ribosome::run_dna(
                &dna.name.clone(),
                context,
                wasm.code.clone(),
                &call,
                Some(call.parameters.into_bytes()),
            )?
        },
        _ => Err(HolochainError::NotImplemented)?,
    };

    if result.is_null() {
        Err(HolochainError::SerializationError(String::from(
            "__hdk_get_validation_package_for_entry_type returned empty result",
        )))
    } else {
        match ValidationPackageDefinition::try_from(result) {
            Ok(package) => Ok(CallbackResult::ValidationPackageDefinition(package)),
            Err(_) => Err(HolochainError::SerializationError(String::from(
                "validation_package result could not be deserialized as ValidationPackage",
            ))),
        }
    }
}

fn wasm_for_entry_type(entry_type: &String, dna: &Dna, context: &Arc<Context>) -> DnaWasm {
    let zome_name = dna.get_zome_name_for_entry_type(entry_type)
        .expect("App entry types must be defined");
    get_wasm(context, &zome_name)
        .expect("Zomes must have a WASM binary")
}

fn find_link_definition_in_dna(base_type: &EntryType, tag: &String, target_type: &EntryType, dna: &Dna, context: &Arc<Context>) -> Option<(LinkValidationPackageArgs, DnaWasm)> {
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
                        wasm_for_entry_type(&app_entry_type, dna, context)
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
                            wasm_for_entry_type(&app_entry_type, dna, context)
                        ))
                    })
            },
            _ => None
        }
    )
}