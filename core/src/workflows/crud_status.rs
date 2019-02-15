use crate::{
    context::Context,
    dht::{actions::hold::hold_entry,dht_store::DhtStore},
    network::{
        actions::get_validation_package::get_validation_package, entry_with_header::EntryWithHeader,
    },
    nucleus::actions::validate::validate_entry,
  
};

use holochain_core_types::{
    cas::content::{Address,AddressableContent},
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
    crud_status::{create_crud_link_eav, create_crud_status_eav, CrudStatus, STATUS_NAME},
    entry::Entry
};
use std::sync::Arc;

pub async fn crud_status_workflow<'a>(
    entry_with_header: &'a EntryWithHeader,
    context: &'a Context,
    crud_status :CrudStatus
) -> Result<(), HolochainError> {
    let EntryWithHeader { entry, header } = &entry_with_header;
    let state = context.state().ok_or(HolochainError::ErrorGeneric("Could not get state".to_string()))?;
    let store = state.clone().dht();
    match crud_status
    {
        CrudStatus::Live => {
            store_entry(&entry,&store)
        },
        CrudStatus::Modified => {
            unimplemented!("MODIFIED NOT IMPLEMENTED")
        },
        CrudStatus::Deleted => {
            unimplemented!("DELETED NOT IMPLEMENTED")
        },
        _ =>
        {
            Err(HolochainError::ErrorGeneric("Crud Status Variant unimplemented".to_string()))
        }
    }

}

fn store_entry(entry:&Entry,dht_store : &DhtStore) ->Result<(),HolochainError>
{
    let live_status = create_crud_status_eav(&entry.address(), CrudStatus::Live)?;
    let store = dht_store.meta_storage().clone();
    let mut meta_storage = store.try_write().map_err(|err|{
        HolochainError::ErrorGeneric("THREAD PROBLEM : Could not get lock from meta storage".to_string())
    })?;
    meta_storage.add_eavi(&live_status)?;
    Ok(())
}

