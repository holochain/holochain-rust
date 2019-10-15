use crate::{
    context::Context, dht::actions::remove_link::remove_link,
    network::entry_with_header::EntryWithHeader, nucleus::validation::validate_entry,
    workflows::hold_entry::hold_entry_workflow,
};

use crate::{
    nucleus::{
        actions::add_pending_validation::add_pending_validation, validation::ValidationError,
    },
    scheduled_jobs::pending_validations::ValidatingWorkflow,
    workflows::validation_package,
};
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    validation::{EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn remove_link_workflow(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    let link_remove = match &entry_with_header.entry {
        Entry::LinkRemove((link_remove, _)) => link_remove,
        _ => Err(HolochainError::ErrorGeneric(
            "remove_link_workflow expects entry to be an Entry::LinkRemove".to_string(),
        ))?,
    };
    let link = link_remove.link().clone();

    log_debug!(context, "workflow/remove_link: {:?}", link);
    // 1. Get hold of validation package
    log_debug!(context,
        "workflow/remove_link: getting validation package..."
    );
    let maybe_validation_package = validation_package(&entry_with_header, context.clone())
        .await
        .map_err(|err| {
            let message = "Could not get validation package from source! -> Add to pending...";
            log_debug!(context, "workflow/remove_link: {}", message);
            log_debug!(context, "workflow/remove_link: Error was: {:?}", err);
            add_pending_validation(
                entry_with_header.to_owned(),
                Vec::new(),
                ValidatingWorkflow::RemoveLink,
                context.clone(),
            );
            HolochainError::ValidationPending
        })?;

    let validation_package = maybe_validation_package
        .ok_or_else(|| "Could not get validation package from source".to_string())?;
    log_debug!(context, "workflow/remove_link: got validation package!");

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta,
    };

    // 3. Validate the entry
    log_debug!(context, "workflow/remove_link: validate...");
    validate_entry(
        entry_with_header.entry.clone(),
        None,
        validation_data,
        &context
    ).await
    .map_err(|err| {
        if let ValidationError::UnresolvedDependencies(dependencies) = &err {
            log_debug!(context, "workflow/remove_link: Link could not be validated due to unresolved dependencies and will be tried later. List of missing dependencies: {:?}", dependencies);
            add_pending_validation(
                entry_with_header.to_owned(),
                dependencies.clone(),
                ValidatingWorkflow::HoldLink,
                context.clone(),
            );
            HolochainError::ValidationPending
        } else {
            log_warn!(context, "workflow/remove_link: Link {:?} is NOT valid! Validation error: {:?}",
                entry_with_header.entry,
                err,
            );
            HolochainError::from(err)
        }

    })?;

    log_debug!(context, "workflow/remove_link: is valid!");

    // 3. If valid store remove the entry in the local DHT shard
    remove_link(&entry_with_header.entry, &context).await?;
    log_debug!(context, "workflow/remove_link: added! {:?}", link);

    //4. store link_remove entry so we have all we need to respond to get links queries without any other network look-up```
    hold_entry_workflow(&entry_with_header, context.clone()).await?;
    log_debug!(
        context,
        "workflow/hold_entry: added! {:?}",
        entry_with_header
    );

    Ok(())
}
