use crate::{
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},

};
use holochain_core_types::{
    entry::{deletion_entry::DeletionEntry, Entry},
    error::HolochainError,
};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use holochain_wasm_types::get_entry::*;
use holochain_wasmer_host::*;
use std::sync::Arc;
use crate::context::Context;

/// ZomeApiFunction::RemoveEntry function code
/// args: [0] encoded MemoryAllocation
/// Expected Address argument
/// Stores/returns a RibosomeReturnValue
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn remove_entry_workflow(
    context: Arc<Context>,
    deleted_entry_address: &Address,
) -> Result<Address, HolochainError> {
    // Get Current entry's latest version
    let get_args = GetEntryArgs {
        address: deleted_entry_address.to_owned(),
        options: Default::default(),
    };
    let maybe_entry_result = get_entry_result_workflow(Arc::clone(&context), &get_args).await;

    if let Err(err) = maybe_entry_result {
        log_error!(context, "zome: get_entry_result_workflow failed: {:?}", err);
        return Err(HolochainError::Wasm(WasmError::WorkflowFailed));
    }

    let entry_result = maybe_entry_result?;
    if !entry_result.found() {
        return Err(HolochainError::Wasm(WasmError::EntryNotFound));
    }
    let deleted_entry_address = entry_result.latest()?.address();

    // Create deletion entry
    let deletion_entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address.clone()));

    author_entry(
            &deletion_entry.clone(),
            Some(deleted_entry_address),
            &context,
            &vec![],
        ).await
        .map(|_| deletion_entry.address())
}
