use crate::{
    context::Context,
    dht::actions::update_entry::update_entry,
    network::{
        actions::get_validation_package::get_validation_package, entry_with_header::EntryWithHeader,
    },
    nucleus::validation::validate_entry,
};
use holochain_persistence_api::cas::content::{Address, AddressableContent};

use holochain_core_types::{
    error::HolochainError,
    validation::{EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn hold_update_workflow<'a>(
    entry_with_header: EntryWithHeader,
    context: Arc<Context>,
) -> Result<Address, HolochainError> {
    let EntryWithHeader { entry, header } = &entry_with_header;

    // 1. Get validation package from source
    let maybe_validation_package = await!(get_validation_package(header.clone(), &context))?;
    let validation_package = maybe_validation_package
        .ok_or("Could not get validation package from source".to_string())?;

    // get link from header
    let link = header
        .link_update_delete()
        .ok_or("Could not get link update from header".to_string())?;

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta,
    };

    // 3. Validate the entry
    await!(validate_entry(
        entry.clone(),
        Some(link.clone()),
        validation_data,
        &context
    ))?;

    // 3. If valid store the entry in the local DHT shard
    await!(update_entry(
        &context.clone(),
        link,
        entry.address().clone()
    ))
}
