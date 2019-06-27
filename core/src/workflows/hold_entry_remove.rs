use crate::{
    context::Context, dht::actions::remove_entry::remove_entry,
    network::entry_with_header::EntryWithHeader, nucleus::validation::validate_entry,
};

use crate::workflows::validation_package;
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    validation::{EntryLifecycle, ValidationData},
};
use holochain_persistence_api::cas::content::AddressableContent;
use std::sync::Arc;

pub async fn hold_remove_workflow(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    // 1. Get hold of validation package
    let maybe_validation_package = await!(validation_package(entry_with_header, context.clone()))?;
    let validation_package = maybe_validation_package
        .ok_or("Could not get validation package from source".to_string())?;

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta,
    };

    // 3. Validate the entry
    await!(validate_entry(
        entry_with_header.entry.clone(),
        None,
        validation_data,
        &context
    ))?;

    let deletion_entry = unwrap_to!(entry_with_header.entry => Entry::Deletion);

    let deleted_entry_address = deletion_entry.clone().deleted_entry_address();
    // 3. If valid store the entry in the local DHT shard
    await!(remove_entry(
        &context.clone(),
        deleted_entry_address,
        entry_with_header.entry.address().clone()
    ))
}
