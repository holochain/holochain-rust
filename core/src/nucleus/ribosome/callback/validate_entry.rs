extern crate serde_json;
use context::Context;
use holochain_core_types::{
    entry::{dna::wasm::DnaWasm, Entry},
    error::HolochainError,
};
use holochain_wasm_utils::validation::ValidationData;
use nucleus::{
    ribosome::{
        self,
        callback::{get_dna, CallbackResult},
    },
    ZomeFnCall,
};
use std::sync::Arc;
use holochain_core_types::entry::dna::zome::ZomeName;
use holochain_core_types::cas::content::AddressableContent;

pub fn validate_entry(
    entry: &Entry,
    validation_data: &ValidationData,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    match entry {
        Entry::App(_, _) => Ok(validate_app_entry(
            entry,
            validation_data,
            context,
        )?),
        Entry::Dna(_) => Ok(CallbackResult::Pass),
        _ => Ok(CallbackResult::NotImplemented),
    }
}

fn validate_app_entry(
    app_entry: &Entry,
    validation_data: &ValidationData,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    match app_entry {
        Entry::App(app_entry_type, _) => {
            let dna = get_dna(&context).expect("Callback called without DNA set!");
            let zome_name = dna.get_zome_name_for_entry_type(&app_entry_type);
            if zome_name.is_none() {
                return Ok(CallbackResult::NotImplemented);
            }

            let zome_name = zome_name.unwrap();
            match get_wasm(&context, &zome_name) {
                Some(wasm) => {
                    let validation_call = build_validation_call(zome_name, app_entry, validation_data)?;
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
        _ => unreachable!(),
    }
}

fn build_validation_call(
    zome_name: ZomeName,
    entry: &Entry,
    validation_data: &ValidationData,
) -> Result<ZomeFnCall, HolochainError> {
    let function_name = format!("validate_{}", entry.entry_type());

    let validation_data_json = serde_json::to_value(&validation_data)
        .expect("ValidationData could not be turned into JSON?!");

    let entry_content = entry.content();

    // Trying to interpret entry as json object
    let serialization_result: Result<serde_json::Value, _> = serde_json::from_str(&entry_content)
        .or_else(|_| {
            // If it can't be parsed as object, treat it as a string by adding quotation marks:
            serde_json::from_str(&format!("\"{}\"", &entry_content))
        })
        .or_else(|error| {
            let msg = format!("Error trying to serialize entry '{}', {:?}", entry_content, error);
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
        Ok(runtime) => match runtime.result.is_empty() {
            true => CallbackResult::Pass,
            false => CallbackResult::Fail(runtime.result),
        },
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
