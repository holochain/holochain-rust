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
    cas::content::{Address, AddressableContent},
    entry::{entry_type::AppEntryType, Entry},
    validation::ValidationPackage,
};
use holochain_wasm_utils::api_serialization::validation::EntryValidationArgs;
use std::sync::Arc;

pub async fn validate_app_entry(
    entry: Entry,
    app_entry_type: AppEntryType,
    context: &Arc<Context>,
    link: Option<Address>,
    validation_package: ValidationPackage,
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set!");

    let zome_name = dna
        .get_zome_name_for_app_entry_type(&app_entry_type)
        .ok_or(ValidationError::NotImplemented)?;

    if link.is_some() {
        let expected_link_update = link
            .clone()
            .expect("Should unwrap link_update_delete with no problems");

        get_entry_from_dht(&context.clone(), &expected_link_update).map_err(|_| {
            ValidationError::Fail("Could not find entry for link_update_delete".to_string())
        })?;

        await!(run_call_back(
            context.clone(),
            entry,
            &zome_name,
            link,
            validation_package
        ))
    } else {
        await!(run_call_back(
            context.clone(),
            entry,
            &zome_name,
            link,
            validation_package
        ))
    }
}

async fn run_call_back(
    context: Arc<Context>,
    entry: Entry,
    zome_name: &String,
    link_update_delete: Option<Address>,
    validation_package: ValidationPackage,
) -> ValidationResult {
    let params = EntryValidationArgs {
        validation_data: entry_to_validation_data(
            context.clone(),
            &entry,
            link_update_delete,
            validation_package,
        )
        .map_err(|_| ValidationError::Fail("Could not get entry validation".to_string()))?,
    };
    let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);

    await!(run_validation_callback(entry.address(), call, &context))
}
