use crate::{
    context::Context,
    holochain_wasm_engine::holochain_persistence_api::cas::content::AddressableContent,
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_types::{get_entry::*, UpdateEntryArgs};
use holochain_wasmer_host::*;
use std::sync::Arc;
use holochain_core_types::error::HolochainError;

/// ZomeApiFunction::UpdateEntry function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: UpdateEntryArgs
/// Returns an HcApiReturnCode as I64
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_update_entry(
    context: Arc<Context>,
    entry_args: UpdateEntryArgs,
) -> Result<Address, HolochainError> {
    // Get Current entry's latest version
    let get_args = GetEntryArgs {
        address: entry_args.address,
        options: Default::default(),
    };
    let maybe_entry_result = context.block_on(get_entry_result_workflow(context, &get_args));
    if let Err(err) = maybe_entry_result {
        log_error!(context, "zome: get_entry_result_workflow failed: {:?}", err);
        Err(WasmError::WorkflowFailed)?;
    }
    let entry_result = maybe_entry_result?.clone();
    if !entry_result.found() {
        Err(WasmError::EntryNotFound)?;
    }
    let latest_entry = entry_result.latest()?;

    // Create Chain Entry
    let entry = entry_args.new_entry.clone();

    context
        .block_on(author_entry(
            &entry,
            Some(latest_entry.address()),
            context,
            &vec![], // TODO should provenance be a parameter?
        ))
        .map(|result| result.address())
}
