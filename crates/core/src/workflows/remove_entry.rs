use crate::{
    context::Context,
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::{
    entry::{deletion_entry::DeletionEntry, Entry},
    error::HolochainError,
};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use holochain_wasm_types::get_entry::*;
use holochain_wasmer_host::*;
use std::sync::Arc;

/// ZomeApiFunction::RemoveEntry function code
/// args: [0] encoded MemoryAllocation
/// Expected Address argument
/// Stores/returns a RibosomeReturnValue
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_remove_entry(
    context: Arc<Context>,
    deleted_entry_address: Address,
) -> Result<Address, HolochainError> {
    // Get Current entry's latest version
    let get_args = GetEntryArgs {
        address: deleted_entry_address,
        options: Default::default(),
    };
    let maybe_entry_result = context.block_on(get_entry_result_workflow(context, &get_args));

    if let Err(err) = maybe_entry_result {
        log_error!(context, "zome: get_entry_result_workflow failed: {:?}", err);
        Err(WasmError::WorkflowFailed)?;
    }

    let entry_result = maybe_entry_result?;
    if !entry_result.found() {
        Err(WasmError::EntryNotFound)?;
    }
    let deleted_entry_address = entry_result.latest()?.address();

    // Create deletion entry
    let deletion_entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address.clone()));

    context
        .block_on(author_entry(
            &deletion_entry.clone(),
            Some(deleted_entry_address),
            context,
            &vec![],
        ))
        .map(|_| deletion_entry.address())
}
