use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        validation::{ValidationError, ValidationResult,entry_to_validation_data},
        CallbackFnCall,
    },
    workflows::get_entry_result::get_entry_result_workflow
};
use holochain_core_types::{
    cas::content::{Address,AddressableContent},
    entry::{entry_type::AppEntryType, Entry}
};
use holochain_wasm_utils::api_serialization::{validation::EntryValidationArgs,get_entry::GetEntryArgs};
use std::sync::Arc;

use futures_util::try_future::TryFutureExt;

pub async fn validate_app_entry(
    entry: Entry,
    app_entry_type: AppEntryType,
    context: &Arc<Context>,
    link : Option<Address>
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set!");

    
    let zome_name = dna
        .get_zome_name_for_app_entry_type(&app_entry_type)
        .ok_or(ValidationError::NotImplemented)?;
    if link.is_some()
    {
        let expected_link_update = link.clone().expect("Should unwrap link_update_delete with no problems");
        let entry_args = &GetEntryArgs {
        address: expected_link_update.clone(),
        options: Default::default()};
        let result = await!(get_entry_result_workflow(&context,entry_args).map_err(|_|{
            ValidationError::Fail("Could not get entry for link_update_delete".to_string())
        }))?;
        result.latest().ok_or(ValidationError::Fail("Could not find entry for link_update_delete".to_string()))?;
        await!(run_call_back(context.clone(), entry, &zome_name, link))
    }
    else 
    {
        await!(run_call_back(context.clone(), entry, &zome_name,link))
    }

    
}

async fn run_call_back(context:Arc<Context>,entry:Entry,zome_name:&String,link_update_delete:Option<Address>)-> ValidationResult
{
    let params = EntryValidationArgs {
        validation_data: entry_to_validation_data(context.clone(),&entry,link_update_delete).map_err(|_|{
            ValidationError::Fail("Could not get entry validation".to_string())
        })?
    };

    let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);

    await!(run_validation_callback(entry.address(), call, &context))
}
