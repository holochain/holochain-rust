use crate::{
    context::Context,
    dht::{actions::crud_status::crud_status as init_crud_future,dht_store::DhtStore},
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
    context: &'a Arc<Context>,
    crud_status :CrudStatus
) -> Result<(), HolochainError> {


    let EntryWithHeader { entry, header } = &entry_with_header;

     // 1. Get validation package from source
    let maybe_validation_package = await!(get_validation_package(header.clone(), &context))?;
    let validation_package = maybe_validation_package
        .ok_or("Could not get validation package from source".to_string())?;

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Dht,
        action: EntryAction::Create,
    };

    // 3. Validate the entry
    await!(validate_entry(entry.clone(), validation_data, &context))?;

    // 4. If valid store the entry in the local DHT shard
    await!(init_crud_future(entry_with_header.clone(), context.clone(),crud_status.clone()))

}

