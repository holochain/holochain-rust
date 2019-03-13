use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        validation::{ValidationError, ValidationResult,entry_to_validation_data},
        CallbackFnCall,
    },
    workflows::get_entry_result::get_entry_result_workflow,
    network::entry_with_header::{EntryWithHeader,fetch_entry_with_header}
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
    _validation_data: ValidationData,
    context: &Arc<Context>) -> ValidationResult
    {
        let dna = context.get_dna().expect("Callback called without DNA set");
        let deletion_entry = unwrap_to!(entry=>Entry::Deletion);
        let EntryWithHeader{entry : entry_to_delete,header: _entity_to_delete_header} = fetch_entry_with_header(&deletion_entry.clone().deleted_entry_address(),&context).map_err(|_|{
            ValidationError::Fail("Author mismatch ".to_string())
        })?;

            let app_entry_type = match entry_to_delete.clone()
            {
            Entry::App(app_entry_type,_) => Ok(app_entry_type),
            _ => Err(ValidationError::Fail("Entry type should be App Type".to_string()))
            }?;

            let zome_name = dna
            .get_zome_name_for_app_entry_type(&app_entry_type)
            .ok_or(ValidationError::NotImplemented)?;
            let entry_args = &GetEntryArgs {
                 address: deletion_entry.clone().deleted_entry_address(),
                      options: Default::default()};
            let result = await!(get_entry_result_workflow(&context,entry_args).map_err(|_|{
                ValidationError::Fail("Could not get entry for link_update_delete".to_string())
            }))?;
            result.latest().ok_or(ValidationError::Fail("Could not find entry for deletion entry".to_string()))?;
            let params = EntryValidationArgs {
            validation_data: entry_to_validation_data(context.clone(),&entry,None).map_err(|_|{
            ValidationError::Fail("Could not get entry validation".to_string())
        })?
            };
            let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);
            await!(run_validation_callback(entry.address(), call, context))
        
    }

