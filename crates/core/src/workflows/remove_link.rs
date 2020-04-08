use crate::{
    context::Context,
    dht::actions::hold_aspect::hold_aspect,
    network::entry_with_header::EntryWithHeader,
    nucleus::validation::{process_validation_err, validate_entry},
    workflows::{hold_entry::hold_entry_workflow, validation_package},
};
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    network::entry_aspect::EntryAspect,
    validation::{EntryLifecycle, ValidationData},
};
use holochain_persistence_api::cas::content::AddressableContent;
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn remove_link_workflow(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    let (link_data, links_to_remove) = match &entry_with_header.entry {
        Entry::LinkRemove(data) => data,
        _ => Err(HolochainError::ErrorGeneric(
            "remove_link_workflow expects entry to be an Entry::LinkRemove".to_string(),
        ))?,
    };
    let link = link_data.link().clone();

    log_debug!(context, "workflow/remove_link: {:?}", link);
    // 1. Get hold of validation package
    log_debug!(
        context,
        "workflow/remove_link: getting validation package..."
    );
    let maybe_validation_package = validation_package(&entry_with_header, context.clone())
        .await
        .map_err(|err| {
            let message = "Could not get validation package from source! -> Add to pending...";
            log_debug!(context, "workflow/remove_link: {}", message);
            log_debug!(context, "workflow/remove_link: Error was: {:?}", err);
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
        &context,
    )
    .await
    .map_err(|err| {
        process_validation_err(
            "remove_link",
            context.clone(),
            err,
            entry_with_header.entry.address(),
        )
    })?;

    log_debug!(context, "workflow/remove_link: is valid!");

    // 3. If valid store the entry aspect in the local DHT shard
    let aspect = EntryAspect::LinkRemove(
        (link_data.clone(), links_to_remove.clone()),
        entry_with_header.header.clone(),
    );
    hold_aspect(aspect, context.clone()).await?;
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
