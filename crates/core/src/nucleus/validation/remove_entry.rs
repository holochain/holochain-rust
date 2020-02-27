use crate::{
    context::Context,
    nucleus::{
        actions::{
            get_entry::get_entry_from_dht, run_validation_callback::run_validation_callback,
        },
        validation::{entry_to_validation_data},
        CallbackFnCall,
    },
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::validation::ValidationResult;
use holochain_core_types::{entry::Entry, validation::ValidationData};
use holochain_persistence_api::cas::content::AddressableContent;
use holochain_wasm_types::validation::EntryValidationArgs;
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn validate_remove_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set");
    let deletion_entry = unwrap_to!(entry=>Entry::Deletion);
    let deletion_address = deletion_entry.deleted_entry_address().clone();

    let entry_to_delete = match get_entry_from_dht(&context.clone(), &deletion_address) {
        Err(_) => return ValidationResult::UnresolvedDependencies(vec![deletion_address.clone()]),
        Ok(None) => return ValidationResult::Fail("Could not obtain entry for link_update_delte".to_string()),
        Ok(Some(v)) => v,
    };

    let app_entry_type = match entry_to_delete.clone() {
        Entry::App(v, _) => v,
        _ => return ValidationResult::Fail("Entry type should be App Type".to_string()),
    };

    let zome_name = match dna.get_zome_name_for_app_entry_type(&app_entry_type) {
        Some(v) => v,
        None => return ValidationResult::NotImplemented,
    };

    let params = EntryValidationArgs {
        validation_data: match entry_to_validation_data(context.clone(), &entry, None, validation_data) {
            Ok(v) => v,
            Err(_) => return ValidationResult::Fail("Could not get entry validation".to_string()),
        },
    };

    let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);
    run_validation_callback(entry.address(), call, context).await
}
