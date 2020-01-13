use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},
};
use holochain_core_types::{
    entry::{deletion_entry::DeletionEntry, Entry},
    error::HolochainError,
};

use holochain_persistence_api::cas::content::{Address, AddressableContent};

use holochain_wasm_utils::api_serialization::get_entry::*;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::RemoveEntry function code
/// args: [0] encoded MemoryAllocation
/// Expected Address argument
/// Stores/returns a RibosomeEncodedValue
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_remove_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;

    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let try_address = Address::try_from(args_str.clone());

    // Exit on error
    if try_address.is_err() {
        log_error!(
            context,
            "zome: invoke_remove_entry failed to deserialize Address: {:?}",
            args_str
        );
        return ribosome_error_code!(ArgumentDeserializationFailed);
    }
    let deleted_entry_address = try_address.unwrap();

    // Get Current entry's latest version
    let get_args = GetEntryArgs {
        address: deleted_entry_address,
        options: Default::default(),
    };
    let maybe_entry_result = context
        .clone()
        .block_on(get_entry_result_workflow(&context, &get_args));

    if let Err(err) = maybe_entry_result {
        log_error!(context, "zome: get_entry_result_workflow failed: {:?}", err);
        return ribosome_error_code!(WorkflowFailed);
    }

    let entry_result = maybe_entry_result.unwrap();
    if !entry_result.found() {
        return ribosome_error_code!(EntryNotFound);
    }
    let deleted_entry_address = entry_result.latest().unwrap().address();

    // Create deletion entry
    let deletion_entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address.clone()));

    let res: Result<Address, HolochainError> = context
        .block_on(author_entry(
            &deletion_entry.clone(),
            Some(deleted_entry_address),
            &context.clone(),
            &vec![],
        ))
        .map(|_| deletion_entry.address());

    runtime.store_result(res)
}
