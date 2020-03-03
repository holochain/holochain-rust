use crate::{
    holochain_wasm_types::holochain_persistence_api::cas::content::AddressableContent,
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},

};
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_types::{get_entry::*, UpdateEntryArgs};
use holochain_wasmer_host::*;
use holochain_core_types::error::HolochainError;
use std::sync::Arc;
use crate::context::Context;
use crate::workflows::WorkflowResult;

/// ZomeApiFunction::UpdateEntry function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: UpdateEntryArgs
/// Returns an HcApiReturnCode as I64
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn update_entry_workflow(
    context: Arc<Context>,
    entry_args: &UpdateEntryArgs,
) -> WorkflowResult<Address> {
    // Get Current entry's latest version
    let get_args = GetEntryArgs {
        address: entry_args.address.to_owned(),
        options: Default::default(),
    };
    let maybe_entry_result = get_entry_result_workflow(Arc::clone(&context), &get_args).await;
    if let Err(err) = maybe_entry_result {
        log_error!(context, "zome: get_entry_result_workflow failed: {:?}", err);
        return Err(HolochainError::Wasm(WasmError::WorkflowFailed));
    }
    let entry_result = maybe_entry_result?.clone();
    if !entry_result.found() {
        return Err(HolochainError::Wasm(WasmError::EntryNotFound));
    }
    let latest_entry = entry_result.latest()?;

    // Create Chain Entry
    let entry = entry_args.new_entry.clone();

    author_entry(
        Arc::clone(&context),
        &entry,
        Some(latest_entry.address()),
        &vec![], // TODO should provenance be a parameter?
    ).await
    .map(|result| result.address())
}
