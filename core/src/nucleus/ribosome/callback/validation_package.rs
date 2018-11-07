extern crate serde_json;
use context::Context;
use holochain_core_types::{
    entry_type::EntryType, error::HolochainError, validation::ValidationPackageDefinition,
};
use nucleus::{
    ribosome::{
        self,
        callback::{get_dna, get_wasm, CallbackResult},
    },
    ZomeFnCall,
};
use std::{convert::TryFrom, sync::Arc};

pub fn get_validation_package_definition(
    entry_type: EntryType,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    match entry_type {
        EntryType::App(app_entry_type) => {
            let dna = get_dna(&context).expect("Callback called without DNA set!");
            let zome_name = dna.get_zome_name_for_entry_type(&app_entry_type);
            if zome_name.is_none() {
                return Ok(CallbackResult::NotImplemented);
            }

            let zome_name = zome_name.unwrap();
            let wasm = get_wasm(&context, &zome_name)
                .ok_or(HolochainError::ErrorGeneric(String::from("no wasm found")))?;

            let result = ribosome::run_dna(
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
            )?;

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
        _ => Err(HolochainError::NotImplemented),
    }
}
