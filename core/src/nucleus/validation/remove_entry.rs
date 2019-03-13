use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        validation::{ValidationError, ValidationResult,entry_to_validation_data},
        CallbackFnCall,
    },
    network::entry_with_header::{EntryWithHeader,fetch_entry_with_header}
};
use holochain_core_types::{
    cas::content::AddressableContent,
    entry::{ Entry},
    validation::ValidationData
};
use holochain_wasm_utils::api_serialization::{validation::EntryValidationArgs};
use std::sync::Arc;


pub async fn validate_remove_entry(entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>) -> ValidationResult
    {
        let dna = context.get_dna().expect("Callback called without DNA set");
        let deletion_entry = unwrap_to!(entry=>Entry::Deletion);
        let EntryWithHeader{entry : entry_to_delete,header: entity_to_delete_header} = fetch_entry_with_header(&deletion_entry.clone().deleted_entry_address(),&context).map_err(|_|{
            ValidationError::Fail("Author ".to_string())
        })?;
        let headers = &validation_data.package.chain_header;

        if headers.provenances().iter().find(|prov| entity_to_delete_header.provenances().iter().find(|prov2|prov.source()==prov2.source()).is_some()).is_some()
        {
            println!("Provenances match");
            let app_entry_type = match entry_to_delete.clone()
            {
            Entry::App(app_entry_type,_) => Ok(app_entry_type),
            _ => Err(ValidationError::Fail("Entry type should be App Type".to_string()))
            }?;

            let zome_name = dna
            .get_zome_name_for_app_entry_type(&app_entry_type)
            .ok_or(ValidationError::NotImplemented)?;
          
            let params = EntryValidationArgs {
            validation_data: entry_to_validation_data(context.clone(),&entry,None).map_err(|_|{
            ValidationError::Fail("Could not get entry validation".to_string())
        })?
            };
            let call = CallbackFnCall::new(&zome_name, "__hdk_validate_app_entry", params);
            await!(run_validation_callback(entry.address(), call, context))
        }
        else
        {
            Err(ValidationError::Fail("Tried to Delete Entry From Different Author".to_string()))
        }

        
    }

