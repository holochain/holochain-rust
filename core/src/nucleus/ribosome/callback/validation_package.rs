extern crate serde_json;
use context::Context;
use holochain_core_types::{entry_type::EntryType, error::HolochainError};
use nucleus::{
    ribosome::{
        self,
        callback::{get_dna, get_wasm, CallbackResult},
    },
    ZomeFnCall,
};
use std::sync::Arc;

pub fn validation_package(
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
            match get_wasm(&context, &zome_name) {
                None => Err(HolochainError::ErrorGeneric(String::from("no wasm found"))),
                Some(wasm) => {
                    match ribosome::api::call(
                        &dna.name.clone(),
                        context,
                        wasm.code.clone(),
                        &ZomeFnCall::new(
                            &zome_name,
                            "no capability, since this is an entry validation call",
                            "__hdk_get_validation_package_for_entry_type",
                            &app_entry_type,
                        ),
                        Some(app_entry_type.into_bytes()),
                    ) {
                        Err(error) => Err(HolochainError::ErrorGeneric(format!("wasmi error: {}", error))),
                        Ok(runtime) => match runtime.result.is_empty() {
                            true => Ok(CallbackResult::NotImplemented),
                            false => {
                                match serde_json::from_str(&runtime.result) {
                                    Ok(package) => Ok(CallbackResult::ValidationPackage(package)),
                                    Err(_) => Err(HolochainError::SerializationError(String::from("validation_package result could not deserialized as ValidationPackage")))
                                }

                            }
                        },
                    }
                }
            }
        }
        _ => Err(HolochainError::NotImplemented),
    }
}
