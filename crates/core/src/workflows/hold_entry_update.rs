use crate::{
    context::Context,
    dht::actions::hold_aspect::hold_aspect,
    network::entry_with_header::EntryWithHeader,
    nucleus::validation::{process_validation_err, validate_entry},
    workflows::validation_package,
};
use holochain_core_types::{
    error::HolochainError,
    network::entry_aspect::EntryAspect,
    validation::{EntryLifecycle, ValidationData},
};
use holochain_persistence_api::cas::content::AddressableContent;
use snowflake::ProcessUniqueId;
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn hold_update_workflow(
    pending_id: &ProcessUniqueId,
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    let EntryWithHeader { entry, header } = entry_with_header;

    // 1. Get hold of validation package
    let maybe_validation_package = validation_package(&entry_with_header, context.clone())
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
    validate_entry(entry.clone(), Some(link.clone()), validation_data, &context)
        .await
        .map_err(|err| {
            process_validation_err(
                "hold_update",
                context.clone(),
                err,
                entry_with_header.entry.address(),
            )
        })?;

    // 4. If valid store the entry aspect in the local DHT shard
    let aspect = EntryAspect::Update(
        entry_with_header.entry.clone(),
        entry_with_header.header.clone(),
    );
    hold_aspect(pending_id, aspect, context.clone()).await?;

    Ok(())
}
