use crate::{
    context::Context,
    dht::actions::remove_link::remove_link,
    nucleus::actions::{build_validation_package::build_validation_package,validate::validate_entry}
};

use holochain_core_types::{
    cas::content::Address,
    entry::Entry,
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn remove_link_workflow<'a>(
    entry: &'a Entry,
    context: &'a Arc<Context>,
) -> Result<(), HolochainError> {

    let link_remove = match entry {
        Entry::LinkAdd(link_remove) => link_remove,
        _ => Err(HolochainError::ErrorGeneric(
            "hold_link_workflow expects entry to be an Entry::LinkRemove".to_string(),
        ))?,
    };
    let link = link_remove.link().clone();

    context.log(format!("debug/workflow/hold_link: {:?}", link));
    // 1. Get validation package from source
    context.log(format!(
        "debug/workflow/hold_link: getting validation package..."
    ));
    let validation_package = await!(build_validation_package(&entry, &context))?;
    context.log(format!("debug/workflow/hold_link: got validation package!"));

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        sources: vec![Address::from("<insert your agent key here>")],
        lifecycle: EntryLifecycle::Chain,
        action: EntryAction::Create,
    };

    // 3. Validate the entry
    context.log(format!("debug/workflow/hold_link: validate..."));
    await!(validate_entry(entry.clone(), validation_data, &context)).map_err(|err| {
        context.log(format!("debug/workflow/hold_link: invalid! {:?}", err));
        err
    })?;
    context.log(format!("debug/workflow/hold_link: is valid!"));

    // 3. If valid store the entry in the local DHT shard
    await!(remove_link(&link, &context))?;
    context.log(format!("debug/workflow/hold_link: added! {:?}", link));
    Ok(())
}