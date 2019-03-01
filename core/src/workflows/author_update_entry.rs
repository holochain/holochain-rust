use crate::{
    agent::actions::commit::commit_entry,
    context::Context,
    network::actions::publish::publish,
    nucleus::actions::{
        build_validation_package::build_validation_package, validate::validate_entry,
    },
};

use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::Entry,
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn author_update_entry<'a>(
    entry: &'a Entry,
    maybe_link_update_delete: Option<Address>,
    context: &'a Arc<Context>,
) -> Result<Address, HolochainError> {
    let address = entry.address();
    context.log(format!(
        "debug/workflow/authoring_entry: {} with content: {:?}",
        address, entry
    ));

    // 1. Build the context needed for validation of the entry
    let validation_package = await!(build_validation_package(&entry, &context))?;
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Chain,
        action: EntryAction::Modify,
    };

    // 2. Validate the entry
    context.log(format!(
        "debug/workflow/authoring_entry/{}: validating...",
        address
    ));
    await!(validate_entry(entry.clone(), validation_data, &context))?;
    context.log(format!("Authoring entry {}: is valid!", address));

    // 3. Commit the entry
    context.log(format!(
        "debug/workflow/authoring_entry/{}: committing...",
        address
    ));
    let addr = await!(commit_entry(
        entry.clone(),
        maybe_link_update_delete.clone(),
        &context
    ))?;
    context.log(format!(
        "debug/workflow/authoring_entry/{}: committed",
        address
    ));

    // 4. Publish the valid entry to DHT. This will call Hold to itself
    //TODO: missing a general public/private sharing check here, for now just
    // using the entry_type can_publish() function which isn't enough

    if entry.entry_type().can_publish() {
        context.log(format!(
            "debug/workflow/authoring_entry/{}: publishing...",
            address
        ));
        await!(publish(addr.clone(), &context))?;
        context.log(format!(
            "debug/workflow/authoring_entry/{}: published!",
            address
        ));
    } else {
        context.log(format!(
            "debug/workflow/authoring_entry/{}: entry is private, no publishing",
            address
        ));
    }
    Ok(addr)
}

