extern crate serde_json;
use context::Context;
use holochain_core_types::{
    entry::Entry, entry_type::EntryType, error::HolochainError, validation::ValidationData,
};
use holochain_dna::wasm::DnaWasm;
use holochain_wasm_utils::api_serialization::validation::EntryValidationArgs;
use nucleus::{
    ribosome::{
        self,
        callback::{get_dna, get_wasm, CallbackResult},
    },
    ZomeFnCall,
};
use std::sync::Arc;

pub fn validate_entry(
    entry: Entry,
    entry_type: EntryType,
    validation_data: ValidationData,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    match entry_type {
        EntryType::App(app_entry_type) => Ok(validate_app_entry(
            entry,
            app_entry_type,
            validation_data,
            context,
        )?),
        EntryType::Dna => Ok(CallbackResult::Pass),
        _ => Ok(CallbackResult::NotImplemented),
    }
}

fn validate_app_entry(
    entry: Entry,
    app_entry_type: String,
    validation_data: ValidationData,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    let dna = get_dna(&context).expect("Callback called without DNA set!");
    let zome_name = dna.get_zome_name_for_entry_type(&app_entry_type);
    if zome_name.is_none() {
        return Ok(CallbackResult::NotImplemented);
    }

    let zome_name = zome_name.unwrap();
    match get_wasm(&context, &zome_name) {
        Some(wasm) => {
            let validation_call =
                build_validation_call(entry, app_entry_type, zome_name, validation_data)?;
            Ok(run_validation_callback(
                context.clone(),
                validation_call,
                &wasm,
                dna.name.clone(),
            ))
        }
        None => Ok(CallbackResult::NotImplemented),
    }
}

fn build_validation_call(
    entry: Entry,
    entry_type: String,
    zome_name: String,
    validation_data: ValidationData,
) -> Result<ZomeFnCall, HolochainError> {
    let params = serde_json::to_string(&EntryValidationArgs {
        entry_type,
        entry: entry.to_string(),
        validation_data,
    }).expect("EntryValidationArgs could not be turned into JSON?!");

    Ok(ZomeFnCall::new(
        &zome_name,
        "no capability, since this is an entry validation call",
        "__hdk_validate_app_entry",
        &params,
    ))
}

fn run_validation_callback(
    context: Arc<Context>,
    fc: ZomeFnCall,
    wasm: &DnaWasm,
    dna_name: String,
) -> CallbackResult {
    match ribosome::run_dna(
        &dna_name,
        context,
        wasm.code.clone(),
        &fc,
        Some(fc.clone().parameters.into_bytes()),
    ) {
        Ok(call_result) => match call_result.is_empty() {
            true => CallbackResult::Pass,
            false => CallbackResult::Fail(call_result),
        },
        // TODO: have "not matching schema" be its own error
        Err(HolochainError::RibosomeFailed(error_string)) => {
            if error_string == "Argument deserialization failed" {
                CallbackResult::Fail(String::from("JSON object does not match entry schema"))
            } else {
                CallbackResult::Fail(error_string)
            }
        }
        Err(error) => CallbackResult::Fail(error.to_string()),
    }
}
