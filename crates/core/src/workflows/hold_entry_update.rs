use crate::{
    context::Context, dht::actions::update_entry::update_entry,
    network::entry_with_header::EntryWithHeader, nucleus::validation::validate_entry,
};
use holochain_persistence_api::cas::content::AddressableContent;

use crate::{
    nucleus::{
        actions::add_pending_validation::add_pending_validation, validation::ValidationError,
    },
    scheduled_jobs::pending_validations::ValidatingWorkflow,
    workflows::validation_package,
};
use holochain_core_types::{
    error::HolochainError,
    validation::{EntryLifecycle, ValidationData}
};
use std::sync::Arc;

pub async fn hold_update_workflow(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    let EntryWithHeader { entry, header } = entry_with_header;
    context.add_flame_guard("hold_update_workflow");
    // 1. Get hold of validation package
    let maybe_validation_package = validation_package(&entry_with_header, context.clone())
        .await
        .map_err(|err| {
            let message = "Could not get validation package from source! -> Add to pending...";
            log_debug!(context, "workflow/hold_update: {}", message);
            log_debug!(context, "workflow/hold_update: Error was: {:?}", err);
            add_pending_validation(
                entry_with_header.to_owned(),
                Vec::new(),
                ValidatingWorkflow::UpdateEntry,
                context.clone(),
            );
            HolochainError::ValidationPending
        })?;
    let validation_package = maybe_validation_package
        .ok_or_else(|| "Could not get validation package from source".to_string())?;

    // get link from header
    let link = header
        .link_update_delete()
        .ok_or_else(|| "Could not get link update from header".to_string())?;

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta,
    };

    // 3. Validate the entry
    validate_entry(
        entry.clone(),
        Some(link.clone()),
        validation_data,
        &context
    ).await
    .map_err(|err| {
        if let ValidationError::UnresolvedDependencies(dependencies) = &err {
            log_debug!(context, "workflow/hold_update: Entry update could not be validated due to unresolved dependencies and will be tried later. List of missing dependencies: {:?}", dependencies);
            add_pending_validation(
                entry_with_header.to_owned(),
                dependencies.clone(),
                ValidatingWorkflow::UpdateEntry,
                context.clone(),
            );
            HolochainError::ValidationPending
        } else {
            log_warn!(context, "workflow/hold_update: Entry update {:?} is NOT valid! Validation error: {:?}",
                entry_with_header.entry,
                err,
            );
            HolochainError::from(err)
        }

    })?;

    // 3. If valid store the entry in the local DHT shard
    update_entry(&context.clone(), link, entry.address().clone()).await?;
    context.end_flame_guard("hold_update_workflow");
    Ok(())
}
