use crate::{
    context::Context, dht::actions::remove_link::remove_link,
    network::entry_with_header::EntryWithHeader, nucleus::validation::validate_entry,
};

use crate::workflows::validation_package;
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    validation::{EntryLifecycle, ValidationData},
};
use std::sync::Arc;
use crate::nucleus::actions::add_pending_validation::add_pending_validation;
use crate::scheduled_jobs::pending_validations::ValidatingWorkflow;
use crate::nucleus::validation::ValidationError;

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

    context.log(format!("debug/workflow/remove_link: {:?}", link));
    // 1. Get hold of validation package
    context.log(format!(
        "debug/workflow/remove_link: getting validation package..."
    ));
    let maybe_validation_package = await!(validation_package(&entry_with_header, context.clone()))
        .map_err(|err| {
            let message = "Could not get validation package from source! -> Add to pending...";
            context.log(format!("debug/workflow/remove_link: {}", message));
            context.log(format!("debug/workflow/remove_link: Error was: {:?}", err));
            add_pending_validation(
                entry_with_header.to_owned(),
                Vec::new(),
                ValidatingWorkflow::RemoveLink,
                context.clone(),
            );
            HolochainError::ValidationPending
        })?;

    let validation_package = maybe_validation_package
        .ok_or("Could not get validation package from source".to_string())?;
    context.log(format!(
        "debug/workflow/remove_link: got validation package!"
    ));

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta,
    };

    // 3. Validate the entry
    context.log(format!("debug/workflow/remove_link: validate..."));
    await!(validate_entry(
        entry_with_header.entry.clone(),
        None,
        validation_data,
        &context
    ))
    .map_err(|err| {
        if let ValidationError::UnresolvedDependencies(dependencies) = &err {
            context.log(format!("debug/workflow/remove_link: Link could not be validated due to unresolved dependencies and will be tried later. List of missing dependencies: {:?}", dependencies));
            add_pending_validation(
                entry_with_header.to_owned(),
                dependencies.clone(),
                ValidatingWorkflow::HoldLink,
                context.clone(),
            );
            HolochainError::ValidationPending
        } else {
            context.log(format!(
                "info/workflow/remove_link: Link {:?} is NOT valid! Validation error: {:?}",
                entry_with_header.entry,
                err,
            ));
            HolochainError::from(err)
        }

    })?;

    context.log(format!("debug/workflow/remove_link: is valid!"));

    // 3. If valid store remove the entry in the local DHT shard
    await!(remove_link(&entry_with_header.entry, &context))?;
    context.log(format!("debug/workflow/remove_link: added! {:?}", link));
    Ok(())
}
