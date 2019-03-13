use crate::{
    context::Context,
    dht::actions::remove_link::remove_link,
    network::{
        actions::get_validation_package::get_validation_package, entry_with_header::EntryWithHeader,
    },
    nucleus::validation::validate_entry
};

use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    validation::{EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn remove_link_workflow<'a>(
    entry_with_header: &'a EntryWithHeader,
    context: &'a Arc<Context>,
) -> Result<(), HolochainError> {
    let EntryWithHeader { entry, header } = &entry_with_header;

    let link_remove = match entry {
        Entry::LinkRemove(link_remove) => link_remove,
        _ => Err(HolochainError::ErrorGeneric(
            "remove_link_workflow expects entry to be an Entry::LinkRemove".to_string(),
        ))?,
    };
    let link = link_remove.link().clone();

    context.log(format!("debug/workflow/remove_link: {:?}", link));
    // 1. Get validation package from source
    context.log(format!(
        "debug/workflow/remove_link: getting validation package..."
    ));
    let maybe_validation_package = await!(get_validation_package(header.clone(), &context))?;
    let validation_package = maybe_validation_package
        .ok_or("Could not get validation package from source".to_string())?;
    context.log(format!(
        "debug/workflow/remove_link: got validation package!"
    ));

  
    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta
    };

    // 3. Validate the entry
    context.log(format!("debug/workflow/remove_link: validate..."));
    await!(validate_entry(entry.clone(),None, validation_data, &context)).map_err(|err| {
        context.log(format!("debug/workflow/remove_link: invalid! {:?}", err));
        err
    })?;
    context.log(format!("debug/workflow/remove_link: is valid!"));

    // 3. If valid store remove the entry in the local DHT shard
    await!(remove_link(&link, &context))?;
    context.log(format!("debug/workflow/remove_link: added! {:?}", link));
    Ok(())
}
