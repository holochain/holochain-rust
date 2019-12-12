use crate::{
    context::Context,
    dht::actions::hold_aspect::hold_aspect,
    network::chain_pair::ChainPair,
    nucleus::validation::{validate_entry, ValidationError},
    workflows::validation_package,
};
use holochain_core_types::{
    error::HolochainError,
    network::entry_aspect::EntryAspect,
    validation::{EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn hold_update_workflow(
    chain_pair: &ChainPair,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    let entry = chain_pair.entry();
    let header = chain_pair.header();

    // 1. Get hold of validation package
    let maybe_validation_package = validation_package(&chain_pair, context.clone())
        .await
        .map_err(|err| {
            let message = "Could not get validation package from source! -> Add to pending...";
            log_debug!(context, "workflow/hold_update: {}", message);
            log_debug!(context, "workflow/hold_update: Error was: {:?}", err);
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
            HolochainError::ValidationPending
        } else {
            log_warn!(context, "workflow/hold_update: Entry update {:?} is NOT valid! Validation error: {:?}",
                chain_pair.entry(),
                err,
            );
            HolochainError::from(err)
        }

    })?;

    // 4. If valid store the entry aspect in the local DHT shard
    let aspect = EntryAspect::Update(chain_pair.entry(), chain_pair.header());
    hold_aspect(aspect, context.clone()).await?;

    Ok(())
}
