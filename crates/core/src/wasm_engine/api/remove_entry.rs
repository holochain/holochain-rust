use crate::{
    wasm_engine::{api::ZomeApiResult, Runtime},
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},
};
use holochain_core_types::{
    entry::{deletion_entry::DeletionEntry, Entry},
    error::HolochainError,
};

use holochain_persistence_api::cas::content::{Address, AddressableContent};

use holochain_wasm_utils::api_serialization::get_entry::*;

/// ZomeApiFunction::RemoveEntry function code
/// args: [0] encoded MemoryAllocation
/// Expected Address argument
/// Stores/returns a RibosomeEncodedValue
pub fn invoke_remove_entry(runtime: &mut Runtime, deleted_entry_address: Address) -> ZomeApiResult {
    // Get Current entry's latest version
    let get_args = GetEntryArgs {
        address: deleted_entry_address,
        options: Default::default(),
    };
    let maybe_entry_result = runtime
        .context()?
        .block_on(get_entry_result_workflow(&runtime.context()?, &get_args));

    if let Err(err) = maybe_entry_result {
        log_error!(
            runtime.context()?,
            "zome: get_entry_result_workflow failed: {:?}",
            err
        );
        return ribosome_error_code!(WorkflowFailed);
    }

    let entry_result = maybe_entry_result.unwrap();
    if !entry_result.found() {
        return ribosome_error_code!(EntryNotFound);
    }
    let deleted_entry_address = entry_result.latest().unwrap().address();

    // Create deletion entry
    let deletion_entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address.clone()));

    let res: Result<Address, HolochainError> = runtime
        .context()?
        .block_on(author_entry(
            &deletion_entry.clone(),
            Some(deleted_entry_address),
            &runtime.context()?,
            &vec![],
        ))
        .map(|_| deletion_entry.address());

    runtime.store_result(res)
}
