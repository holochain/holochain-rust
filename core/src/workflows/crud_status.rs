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
    eav::{Attribute, EaviQuery, EntityAttributeValueIndex, IndexFilter},
    entry::Entry
};
use std::{sync::Arc,collections::BTreeSet,convert::TryFrom, str::FromStr};


pub async fn crud_status_workflow<'a>(
     context: &'a Arc<Context>,
    address: &'a Address,
    crud_status :&'a CrudStatus
) -> Result<(), HolochainError> 
{

    //create crud_status passed in
    match crud_status
    {
        CrudStatus::Live => 
        {
           store_crud_status(context,address)
        }
        CrudStatus::Modified =>
        {
            update_crud_status(context,address)
        },
        CrudStatus::Deleted => 
        {
            remove_crud_status(context,address)
        }
        _ => Err(HolochainError::NotImplemented("Crud Status Not Implemented".to_string()))
    }

    
   
}

pub async fn crud_link_workflow<'a>(
     context: &'a Arc<Context>,
    address: &'a Address,
    crud_link : &'a Option<Address>
) -> Result<(), HolochainError> 
{

let state = context.state().ok_or(HolochainError::ErrorGeneric("Could not find state".to_string()))?;

    //grab meta from state
    let dht = state.dht().clone();
    let dht_meta = dht.meta_storage().clone();
    //grab lock from meta_storage
    let mut meta_storage = dht_meta.try_write().map_err(|_|HolochainError::ErrorGeneric("Could not get lock".to_string()))?;

    let crud_link = create_crud_link_eav(address, &crud_link.clone().ok_or(HolochainError::ErrorGeneric("CrudLink Not Available".to_string()))?)?;
    meta_storage.add_eavi(&crud_link)?;
    Ok(())
      
}


fn store_crud_status<'a>( context: &'a Arc<Context>,
    address: &'a Address) -> Result<(), HolochainError> 
{
         //grab state from context
    let state = context.state().ok_or(HolochainError::ErrorGeneric("Could not find state".to_string()))?;

    //grab meta from state
    let dht = state.dht().clone();
    let dht_meta = dht.meta_storage().clone();
    //grab lock from meta_storage
    let mut meta_storage = dht_meta.try_write().map_err(|_|HolochainError::ErrorGeneric("Could not get lock".to_string()))?;

    let status = create_crud_status_eav(address, CrudStatus::Live)?;
    meta_storage.add_eavi(&status)?;
    Ok(())
}


fn update_crud_status<'a>( context: &'a Arc<Context>,
    address: &'a Address) -> Result<(), HolochainError> 
{
    //grab state from context
    let state = context.state().ok_or(HolochainError::ErrorGeneric("Could not find state".to_string()))?;

    //grab meta from state
    let dht = state.dht().clone();
    let dht_meta = dht.meta_storage().clone();
    //grab lock from meta_storage
    let mut meta_storage = dht_meta.try_write().map_err(|_|HolochainError::ErrorGeneric("Could not get lock".to_string()))?;

    let status = create_crud_status_eav(address, CrudStatus::Modified)?;
    meta_storage.add_eavi(&status)?;
    Ok(())
}



fn remove_crud_status<'a>( context: &'a Arc<Context>,
    address: &'a Address) -> Result<(), HolochainError> 
    {

     //grab state from context
     let state = context.state().ok_or(HolochainError::ErrorGeneric("Could not find state".to_string()))?;

    //grab meta from state
    let dht = state.dht().clone();
    let dht_meta = dht.meta_storage().clone();
    let dht_content  = dht.content_storage().clone();
    //grab lock from meta_storage
    let mut meta_storage = dht_meta.try_write().map_err(|_|HolochainError::ErrorGeneric("Could not get lock".to_string()))?;

    let content_storage = dht_content.try_read().map_err(|_|HolochainError::ErrorGeneric("Could not get lock".to_string()))?;
        let maybe_json_entry = content_storage
        .fetch(address)
        .unwrap();
        let json_entry = maybe_json_entry.ok_or_else(|| {
        HolochainError::ErrorGeneric(String::from("trying to remove a missing entry"))
        })?;

        let entry = Entry::try_from(json_entry).expect("Stored content should be a valid entry.");
        // pre-condition: entry_type must not by sys type, since they cannot be deleted
        if entry.entry_type().to_owned().is_sys() {
        return Err(HolochainError::ErrorGeneric(String::from(
            "trying to remove a system entry type",
        )));
        }

        // pre-condition: Current status must be Live
    // get current status

    let maybe_status_eav = meta_storage.fetch_eavi(&EaviQuery::new(
        Some(address.clone()).into(),
        Some(Attribute::CrudStatus).into(),
        None.into(),
        IndexFilter::LatestByAttribute,
    ));
    if let Err(err) = maybe_status_eav {
        return Err(err);
    }
    let status_eavs = maybe_status_eav.unwrap();
    assert!(!status_eavs.is_empty(), "Entry should have a Status");
    // TODO waiting for update/remove_eav() assert!(status_eavs.len() <= 1);
    // For now checks if crud-status other than Live are present
    let status_eavs = status_eavs
        .into_iter()
        .filter(|e| CrudStatus::from_str(String::from(e.value()).as_ref()) != Ok(CrudStatus::Live))
        .collect::<BTreeSet<EntityAttributeValueIndex>>();
    if !status_eavs.is_empty() {
        return Err(HolochainError::ErrorGeneric(String::from(
            "entry_status != CrudStatus::Live",
        )));
    }
    // Update crud-status
    let new_status_eav = create_crud_status_eav(address, CrudStatus::Deleted)?;
    meta_storage.add_eavi(&new_status_eav)?;

    Ok(())
    
    }

    fn remove_crud_link<'a>( context: &'a Arc<Context>,
    address: &'a Address,crud_link :&'a Option<Address>) -> Result<(), HolochainError> 
    {

     //grab state from context
     let state = context.state().ok_or(HolochainError::ErrorGeneric("Could not find state".to_string()))?;

    //grab meta from state
    let dht = state.dht().clone();
    let dht_meta = dht.meta_storage().clone();
    let dht_content  = dht.content_storage().clone();
    //grab lock from meta_storage
    let mut meta_storage = dht_meta.try_write().map_err(|_|HolochainError::ErrorGeneric("Could not get lock".to_string()))?;

    let content_storage = dht_content.try_read().map_err(|_|HolochainError::ErrorGeneric("Could not get lock".to_string()))?;
        let maybe_json_entry = content_storage
        .fetch(address)
        .unwrap();
        let json_entry = maybe_json_entry.ok_or_else(|| {
        HolochainError::ErrorGeneric(String::from("trying to remove a missing entry"))
        })?;

        let entry = Entry::try_from(json_entry).expect("Stored content should be a valid entry.");
        // pre-condition: entry_type must not by sys type, since they cannot be deleted
        if entry.entry_type().to_owned().is_sys() {
        return Err(HolochainError::ErrorGeneric(String::from(
            "trying to remove a system entry type",
        )));
        }

        // pre-condition: Current status must be Live
    // get current status

    let maybe_status_eav = meta_storage.fetch_eavi(&EaviQuery::new(
        Some(address.clone()).into(),
        Some(Attribute::CrudStatus).into(),
        None.into(),
        IndexFilter::LatestByAttribute,
    ));
    if let Err(err) = maybe_status_eav {
        return Err(err);
    }
    let status_eavs = maybe_status_eav.unwrap();
    assert!(!status_eavs.is_empty(), "Entry should have a Status");
    // TODO waiting for update/remove_eav() assert!(status_eavs.len() <= 1);
    // For now checks if crud-status other than Live are present
    let status_eavs = status_eavs
        .into_iter()
        .filter(|e| CrudStatus::from_str(String::from(e.value()).as_ref()) != Ok(CrudStatus::Live))
        .collect::<BTreeSet<EntityAttributeValueIndex>>();
    if !status_eavs.is_empty() {
        return Err(HolochainError::ErrorGeneric(String::from(
            "entry_status != CrudStatus::Live",
        )));
    }
    // Update crud-status
    let new_status_eav = create_crud_status_eav(address, CrudStatus::Deleted)?;
    meta_storage.add_eavi(&new_status_eav)?;
    Ok(())
    
    }


