use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        validation::{ValidationError, ValidationResult,entry_to_validation_data},
        CallbackFnCall,
    },
    network::entry_with_header::fetch_entry_with_header
};
use holochain_core_types::{
    cas::content::{Address,AddressableContent},
    entry::{entry_type::AppEntryType, Entry},
    validation::ValidationPackage
};
use holochain_wasm_utils::api_serialization::{validation::EntryValidationArgs};
use std::sync::Arc;



pub async fn validate_app_entry(
    entry: Entry,
    app_entry_type: AppEntryType,
    context: &Arc<Context>,
    link : Option<Address>,
    validation_package : ValidationPackage
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set!");

    
    let zome_name = dna
        .get_zome_name_for_app_entry_type(&app_entry_type)
        .ok_or(ValidationError::NotImplemented)?;
    if link.is_some()
    {
        let expected_link_update = link.clone().expect("Should unwrap link_update_delete with no problems");
        fetch_entry_with_header(&expected_link_update, &context.clone()).map_err(|_|{
            ValidationError::Fail("Could not find entry for link_update_delete".to_string())
        })?;
        let params = EntryValidationArgs {
        validation_data: entry_to_validation_data(context.clone(),&entry,link_update_delete,validation_package).map_err(|_|{
            ValidationError::Fail("Could not get entry validation".to_string())
        })?
        };

        let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);

        await!(run_validation_callback(entry.address(), call, &context))
    }
    else 
    {
        let params = EntryValidationArgs {
        validation_data: entry_to_validation_data(context.clone(),&entry,link_update_delete,validation_package).map_err(|_|{
            ValidationError::Fail("Could not get entry validation".to_string())
        })?
        };

        let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);

        await!(run_validation_callback(entry.address(), call, &context))
    }

    
}


