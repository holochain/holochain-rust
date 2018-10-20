extern crate serde_json;
use context::Context;
use holochain_core_types::{entry::Entry, entry_type::EntryType, error::HolochainError};
use holochain_dna::wasm::DnaWasm;
use holochain_wasm_utils::api_serialization::validation::ValidationData;
use nucleus::{
    ribosome::{
        self,
        callback::{get_dna, CallbackResult},
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
    let function_name = format!("validate_{}", entry_type.to_string());

    let validation_data_json = serde_json::to_value(&validation_data)
        .expect("ValidationData could not be turned into JSON?!");

    // Trying to interpret entry as json object
    let serialization_result: Result<serde_json::Value, _> = serde_json::from_str(&String::from((*entry).to_owned()))
        .or_else(|_| {
            // If it can't be parsed as object, treat it as a string by adding quotation marks:
            serde_json::from_str(&format!("\"{}\"", &*entry))
        })
        .or_else(|error| {
            let msg = format!("Error trying to serialize entry '{}', {:?}", *entry, error);
            Err(HolochainError::new(&msg))
        });

    serialization_result.and_then(|entry_json| {
        let params = serde_json::to_string(&json!({
            "entry": entry_json,
            "ctx": validation_data_json,
        })).expect("Params object could not be turned into JSON?!");

        Ok(ZomeFnCall::new(
            &zome_name,
            "no capability, since this is an entry validation call",
            &function_name,
            &params,
        ))
    })
}

fn run_validation_callback(
    context: Arc<Context>,
    fc: ZomeFnCall,
    wasm: &DnaWasm,
    app_name: String,
) -> CallbackResult {
    match ribosome::api::call(
        &app_name,
        context,
        wasm.code.clone(),
        &fc,
        Some(fc.clone().parameters.into_bytes()),
    ) {
        Ok(runtime) => CallbackResult::from(runtime.result),
        Err(_) => CallbackResult::NotImplemented,
    }
}

fn get_wasm(context: &Arc<Context>, zome: &str) -> Option<DnaWasm> {
    let dna = get_dna(context).expect("Callback called without DNA set!");
    dna.get_wasm_from_zome_name(zome).and_then(|wasm| {
        if wasm.code.len() > 0 {
            Some(wasm.clone())
        } else {
            None
        }
    })
}
