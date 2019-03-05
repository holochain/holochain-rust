use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        validation::{ValidationError, ValidationResult},
        CallbackFnCall,
    },
    workflows::get_entry_result::get_entry_result_workflow
};
use holochain_core_types::{
    cas::content::AddressableContent,
    entry::{ Entry},
    validation::ValidationData
};
use holochain_wasm_utils::api_serialization::{validation::EntryValidationArgs,get_entry::GetEntryArgs};
use std::sync::Arc;
use futures_util::try_future::TryFutureExt;

pub async fn validate_remove_entry(entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>) -> ValidationResult
    {
        let dna = context.get_dna().expect("Callback called without DNA set");
        let deletion_entry = unwrap_to!(entry=>Entry::Deletion);
        let entry_args = GetEntryArgs {
            address: deletion_entry.clone().deleted_entry_address(),
            options: Default::default(),
        };
        let entry_result = await!(get_entry_result_workflow(context,&entry_args).map_err(|_|{
            ValidationError::Fail("Entry not found in dht chain".to_string())
        }))?;

        let entry_to_delete = entry_result.latest().ok_or(ValidationError::Fail("Could not get entry from DHT".to_string()))?;
        let app_entry_type = match entry_to_delete
        {
            Entry::App(app_entry_type,_) => Ok(app_entry_type),
            _ => Err(ValidationError::Fail("Entry type should be App Type".to_string()))
        }?;
        let zome_name = dna
        .get_zome_name_for_app_entry_type(&app_entry_type)
        .ok_or(ValidationError::NotImplemented)?;

        let params = EntryValidationArgs {
        entry: entry.clone(),
        entry_type: entry.entry_type(),
        validation_data: validation_data.clone(),
        };

        let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);
        await!(run_validation_callback(entry.address(), call, context))
    }