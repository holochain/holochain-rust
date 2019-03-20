use crate::{
    context::Context,
    nucleus::{
        actions::{
            get_entry::get_entry_from_dht, run_validation_callback::run_validation_callback,
        },
        validation::{entry_to_validation_data, ValidationError, ValidationResult},
        CallbackFnCall,
    },
};
use holochain_core_types::{
    cas::content::AddressableContent, entry::Entry, validation::ValidationData,
};
use holochain_wasm_utils::api_serialization::validation::EntryValidationArgs;
use std::sync::Arc;

pub async fn validate_remove_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set");
    let deletion_entry = unwrap_to!(entry=>Entry::Deletion);
    let deletion_address = deletion_entry.clone().deleted_entry_address();
    let entry_to_delete = get_entry_from_dht(&context.clone(), &deletion_address)
        .map_err(|_| ValidationError::UnresolvedDependencies(vec![deletion_address.clone()]))?
        .ok_or(ValidationError::Fail(
            "Could not obtain entry for link_update_delte".to_string(),
        ))?;
    let app_entry_type = match entry_to_delete.clone() {
        Entry::App(app_entry_type, _) => Ok(app_entry_type),
        _ => Err(ValidationError::Fail(
            "Entry type should be App Type".to_string(),
        )),
    }?;

    let zome_name = dna
        .get_zome_name_for_app_entry_type(&app_entry_type)
        .ok_or(ValidationError::NotImplemented)?;

    let params = EntryValidationArgs {
        validation_data: entry_to_validation_data(
            context.clone(),
            &entry,
            None,
            validation_data,
        )
        .map_err(|_| ValidationError::Fail("Could not get entry validation".to_string()))?,
    };

    let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);
    await!(run_validation_callback(entry.address(), call, context))
}
