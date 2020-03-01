use crate::{
    context::Context,
    nucleus::{
        actions::{
            get_entry::get_entry_from_dht, run_validation_callback::run_validation_callback,
        },
        validation::{entry_to_validation_data},
        CallbackFnCall,
    },
    
};
use holochain_core_types::{
    entry::{entry_type::AppEntryType, Entry},
    validation::ValidationData,
    validation::{ValidationResult},
};
use holochain_persistence_api::cas::content::{Address, AddressableContent};

use holochain_wasm_types::validation::EntryValidationArgs;
use std::sync::Arc;

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn validate_app_entry(
    context: Arc<Context>,
    entry: Entry,
    app_entry_type: AppEntryType,
    link: Option<Address>,
    validation_data: ValidationData,
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set!");

    let zome_name = match dna.get_zome_name_for_app_entry_type(&app_entry_type) {
        Some(v) => v,
        None => return ValidationResult::NotImplemented,
    };

    if let Some(expected_link_update) = link.clone() {
        if let Err(_) = get_entry_from_dht(&context.clone(), &expected_link_update) {
            return ValidationResult::UnresolvedDependencies(vec![expected_link_update.clone()]);
        };
    };

    let params = EntryValidationArgs {
        validation_data: match entry_to_validation_data(context.clone(), &entry, link, validation_data) {
            Ok(v) => v,
            Err(_) => return ValidationResult::Fail("Could not get entry validation".to_string()),
        },
    };
    let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);

    run_validation_callback(Arc::clone(&context), entry.address(), call).await
}
