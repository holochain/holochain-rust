use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        state::{ValidationError, ValidationResult},
        ZomeFnCall,
    },
};
use holochain_core_types::{
    cas::content::AddressableContent,
    entry::{entry_type::AppEntryType, Entry},
    validation::ValidationData,
};
use holochain_wasm_utils::api_serialization::validation::EntryValidationArgs;
use std::sync::Arc;

pub async fn validate_app_entry(
    entry: Entry,
    app_entry_type: AppEntryType,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set!");
    let zome_name = dna
        .get_zome_name_for_app_entry_type(&app_entry_type)
        .ok_or(ValidationError::NotImplemented)?;

    let params = EntryValidationArgs {
        entry: entry.clone(),
        entry_type: entry.entry_type(),
        validation_data: validation_data.clone(),
    };

    let zome_call = ZomeFnCall::new(&zome_name, None, "__hdk_validate_app_entry", params);

    await!(run_validation_callback(entry.address(), zome_call, context))
}
