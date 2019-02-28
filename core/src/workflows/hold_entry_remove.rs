use crate::{
    context::Context,
    dht::actions::remove_entry::remove_entry,
    network::{
        actions::get_validation_package::get_validation_package, entry_with_header::EntryWithHeader,
    },
    nucleus::actions::validate::validate_entry,
};

use holochain_core_types::{
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
    entry::Entry,
    cas::content::AddressableContent
};
use std::sync::Arc;

pub async fn hold_remove_workflow<'a>(
    entry_with_header: EntryWithHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    let EntryWithHeader { entry, header } = &entry_with_header;
    println!("get validation from source");
    // 1. Get validation package from source
    let maybe_validation_package = await!(get_validation_package(header.clone(), &context))?;
    let validation_package = maybe_validation_package
        .ok_or("Could not get validation package from source".to_string())?;
    println!("create validation data");
    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Dht,
        action: EntryAction::Create,
    };

    println!("validate entry");
    // 3. Validate the entry
    await!(validate_entry(entry.clone(), validation_data, &context))?;


    let deletion_entry = unwrap_to!(entry => Entry::Deletion);
    println!("remove from link");
    let deleted_entry_address = deletion_entry.clone().deleted_entry_address();
    // 3. If valid store the entry in the local DHT shard
    await!(remove_entry(&context.clone(),deletion_entry.clone().deleted_entry_address(),entry.address().clone())?)
}


