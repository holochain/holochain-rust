use crate::{
    context::Context, dht::actions::hold_aspect::hold_aspect,
    network::entry_header_pair::EntryHeaderPair, nucleus::validation::validate_entry,
};

use crate::{nucleus::validation::ValidationError, workflows::validation_package};
use holochain_core_types::{
    error::HolochainError,
    network::entry_aspect::EntryAspect,
    validation::{EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn hold_remove_workflow(
    entry_header_pair: &EntryHeaderPair,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    // 1. Get hold of validation package
    let maybe_validation_package = validation_package(entry_header_pair, context.clone())
        .await
        .map_err(|err| {
            let message = "Could not get validation package from source! -> Add to pending...";
            log_debug!(context, "workflow/hold_remove: {}", message);
            log_debug!(context, "workflow/hold_remove: Error was: {:?}", err);
            HolochainError::ValidationPending
        })?;
    let validation_package = maybe_validation_package
        .ok_or_else(|| "Could not get validation package from source".to_string())?;

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta,
    };

    // 3. Validate the entry
    validate_entry(
        entry_header_pair.entry(),
        None,
        validation_data,
        &context
    ).await
    .map_err(|err| {
        if let ValidationError::UnresolvedDependencies(dependencies) = &err {
            log_debug!(context, "workflow/hold_remove: Entry removal could not be validated due to unresolved dependencies and will be tried later. List of missing dependencies: {:?}", dependencies);
            HolochainError::ValidationPending
        } else {
            log_warn!(context, "workflow/hold_remove: Entry removal {:?} is NOT valid! Validation error: {:?}",
                entry_header_pair.entry(),
                err,
            );
            HolochainError::from(err)
        }

    })?;

    // 4. If valid store the entry aspect in the local DHT shard
    let aspect = EntryAspect::Deletion(entry_header_pair.header());
    hold_aspect(aspect, context.clone()).await?;
    Ok(())
}
