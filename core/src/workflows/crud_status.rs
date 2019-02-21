use crate::{
    context::Context,
    dht::{dht_store::DhtStore},
    network::{
        actions::get_validation_package::get_validation_package, entry_with_header::EntryWithHeader,
    },
    nucleus::actions::validate::validate_entry,
  
};

use holochain_core_types::{
    cas::content::{Address,AddressableContent},
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
    crud_status::{create_crud_link_eav, create_crud_status_eav, CrudStatus},
    entry::Entry
};
use std::sync::Arc;

pub async fn crud_status_workflow<'a>(
     context: &'a Arc<Context>,
    address: &'a Address,
    crud_status :&'a CrudStatus
) -> Result<(), HolochainError> 
{

     //grab state from context
    let state = context.state().ok_or(HolochainError::ErrorGeneric("Could not find state".to_string()))?;

    //grab meta from state
    let dht = state.dht().clone();
    let dht_meta = dht.meta_storage().clone();
    //grab lock from meta_storage
    let mut meta_storage = dht_meta.try_write().map_err(|_|HolochainError::ErrorGeneric("Could not get lock".to_string()))?;
    
    //create crud_status passed in
    let status = create_crud_status_eav(address, *crud_status)?;

    //add status to eavi
    meta_storage.add_eavi(&status)?;
    Ok(()) 
}

