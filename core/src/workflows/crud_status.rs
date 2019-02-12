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

pub async fn crud_workflow<'a>(
    entry_with_header: EntryWithHeader,
    context: Arc<Context>,
    old_store : Arc<DhtStore>,
    crud_status : CrudStatus
) -> Result<Option<DhtStore>, HolochainError> {
    let EntryWithHeader { entry, header } = &entry_with_header;

    match crud_status
    {
        CrudStatus::Live => {
            store_entry(context,&old_store,&entry)
        },
        CrudStatus::Modified => {
            modify_entry(context,&old_store,&entry)
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

pub fn modify_entry(context: Arc<Context>,
    old_store: &DhtStore,
    entry: &Entry) ->Result<Option<DhtStore>,HolochainError>
{
    let new_status_eav = create_crud_status_eav(latest_old_address, CrudStatus::Modified)?;
    Ok((*meta_storage.write().unwrap()).add_eavi(&new_status_eav)
              .map(|_| None)
                .map_err(|err| {
                    closure_store
                        .clone()
                        .actions_mut()
                        .insert(action_wrapper.clone(), Err(err));
                    Some(closure_store.clone())
                })
                .ok()
                .unwrap_or(Some(closure_store.clone())))
              

}
pub fn store_entry(context: Arc<Context>,
    old_store: &DhtStore,
    entry: &Entry) ->Result<Option<DhtStore>,HolochainError>
{
    // Add it to local storage
    let new_store = (*old_store).clone();
    let content_storage = &new_store.content_storage().clone();
    let res = (*content_storage.write().unwrap()).add(entry).map(|err|{
        context.log(format!(
            "err/dht: dht::reduce_hold_entry() FAILED {:?}",
            err
        ));
        err
    })?;
    let meta_storage = new_store.meta_storage().clone();
    let status_eav = create_crud_status_eav(&entry.address(), CrudStatus::Live)?;
    Ok((meta_storage.write().unwrap()).add_eavi(&status_eav)
                    .map(|_| Some(new_store))
                    .map_err(|err| {
                        context.log(format!(
                            "err/dht: reduce_hold_entry: meta_storage write failed!: {:?}",
                            err
                        ));
                        err
                    })?)
  
}