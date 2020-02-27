use crate::{
    context::Context, dht::actions::hold_aspect::hold_aspect,
    network::entry_with_header::EntryWithHeader, nucleus::validation::validate_entry,
    NEW_RELIC_LICENSE_KEY,
};

use crate::{workflows::validation_package};
use holochain_core_types::{
    error::HolochainError,
    network::entry_aspect::EntryAspect,
    validation::{EntryLifecycle, ValidationData, ValidationResult},
};
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn hold_remove_workflow(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    // 1. Get hold of validation package
    let maybe_validation_package = validation_package(entry_with_header, context.clone())
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
    match validate_entry(
        entry_with_header.entry.clone(),
        None,
        validation_data,
        &context
    ).await {
        ValidationResult::Ok => (),
        ValidationResult::UnresolvedDependencies(dependencies) => {
            log_debug!(context, "workflow/hold_remove: Entry removal could not be validated due to unresolved dependencies and will be tried later. List of missing dependencies: {:?}", dependencies);
            return Err(HolochainError::ValidationPending);
        },
        ValidationResult::Fail(e) => {
            log_warn!(context, "workflow/hold_remove: Entry removal {:?} is NOT valid! Validation error: {:?}",
                entry_with_header.entry,
                e,
            );
            return Err(HolochainError::from(e));
        }
    };

    // 4. If valid store the entry aspect in the local DHT shard
    let aspect = EntryAspect::Deletion(entry_with_header.header.clone());
    hold_aspect(aspect, context.clone()).await?;
    Ok(())
}
