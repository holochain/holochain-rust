use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},
};
use holochain_core_types::error::HolochainError;

use holochain_persistence_api::cas::content::{Address, AddressableContent};

use holochain_wasm_utils::api_serialization::{get_entry::*, UpdateEntryArgs};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::UpdateEntry function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: UpdateEntryArgs
/// Returns an HcApiReturnCode as I64
pub fn invoke_update_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let entry_args = match UpdateEntryArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            log_error!(
                context,
                "zome: invoke_update_entry failed to deserialize SerializedEntry: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    // Get Current entry's latest version
    let get_args = GetEntryArgs {
        address: entry_args.address,
        options: Default::default(),
    };
    let maybe_entry_result = context.block_on(get_entry_result_workflow(&context, &get_args));
    if let Err(err) = maybe_entry_result {
        log_error!(context, "zome: get_entry_result_workflow failed: {:?}", err);
        return ribosome_error_code!(WorkflowFailed);
    }
    let entry_result = maybe_entry_result.clone().unwrap();
    if !entry_result.found() {
        return ribosome_error_code!(EntryNotFound);
    }
    let latest_entry = entry_result.latest().unwrap();

    // Create Chain Entry
    let entry = entry_args.new_entry.clone();

    let res: Result<Address, HolochainError> = context
        .block_on(author_entry(
            &entry,
            Some(latest_entry.clone().address()),
            &context.clone(),
            &vec![], // TODO should provenance be a parameter?
        ))
        .map(|result| result.address());

    runtime.store_result(res)
}
