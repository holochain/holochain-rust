use crate::{
    agent::actions::commit::commit_entry,
    dht::actions::remove_entry::remove_entry,
    nucleus::{
        actions::{build_validation_package::*, validate::*},
        ribosome::{api::ZomeApiResult, Runtime},
    },
};
use futures::{
    executor::block_on,
    future::{self, TryFutureExt},
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::{deletion_entry::DeletionEntry, Entry},
    error::HolochainError,
    hash::HashString,
    validation::{EntryAction, EntryLifecycle, ValidationData},
};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::RemoveEntry function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected Address argument
/// Returns only a RibosomeReturnCode as I32
pub fn invoke_remove_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let try_address = Address::try_from(args_str.clone());
    // Exit on error
    if try_address.is_err() {
        println!(
            "invoke_remove_entry failed to deserialize Address: {:?}",
            args_str
        );
        return ribosome_error_code!(ArgumentDeserializationFailed);
    }
    let deleted_entry_address = try_address.unwrap();
    // Create deletion entry
    let deletion_entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address.clone()));
    // Resolve future
    let result: Result<(), HolochainError> = block_on(
        // 1. Build the context needed for validation of the entry
        build_validation_package(&deletion_entry, &runtime.context)
            .and_then(|validation_package| {
                future::ready(Ok(ValidationData {
                    package: validation_package,
                    sources: vec![HashString::from("<insert your agent key here>")],
                    lifecycle: EntryLifecycle::Chain,
                    action: EntryAction::Delete,
                }))
            })
            // 2. Validate the entry
            .and_then(|validation_data| {
                validate_entry(deletion_entry.clone(), validation_data, &runtime.context)
            })
            // 3. Commit the valid entry to chain and DHT
            .and_then(|_| {
                commit_entry(
                    deletion_entry.clone(),
                    Some(deleted_entry_address.clone()),
                    &runtime.context,
                )
            })
            // 4. Remove the entry in DHT metadata
            .and_then(|_| {
                remove_entry(
                    &runtime.context,
                    &runtime.context.action_channel,
                    deleted_entry_address.clone(),
                    deletion_entry.address().clone(),
                )
            }),
    );
    // Done
    match result {
        Err(_) => ribosome_error_code!(Unspecified),
        Ok(_) => ribosome_success!(),
    }
}
