use crate::{
    agent::actions::commit::commit_entry,
    nucleus::{ribosome::{api::ZomeApiResult, Runtime},
    actions::{build_validation_package::*, validate::*}},
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},
};
use futures::future::{self, TryFutureExt};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
    entry::Entry
};
use holochain_wasm_utils::api_serialization::{get_entry::*, UpdateEntryArgs};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::UpdateEntry function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: UpdateEntryArgs
/// Returns an HcApiReturnCode as I64
pub fn invoke_update_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let zome_call_data = runtime.zome_call_data()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let entry_args = match UpdateEntryArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            zome_call_data.context.log(format!(
                "err/zome: invoke_update_entry failed to deserialize SerializedEntry: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    // Get Current entry's latest version
    let get_args = GetEntryArgs {
        address: entry_args.address,
        options: Default::default(),
    };
    let maybe_entry_result = zome_call_data.context.block_on(get_entry_result_workflow(
        &zome_call_data.context,
        &get_args,
    ));
    if let Err(_err) = maybe_entry_result {
        return ribosome_error_code!(Unspecified);
    }
    let entry_result = maybe_entry_result.unwrap();
    if !entry_result.found() {
        return ribosome_error_code!(Unspecified);
    }
    let latest_entry = entry_result.latest().unwrap();

    // Get latest entry's ChainHeader
    let agent_state = &zome_call_data.context.state().unwrap().agent();
    let chain_header_address = agent_state
        .chain_store()
        .iter(&agent_state.top_chain_header())
        .find(|header| header.entry_address() == &latest_entry.address())
        .map(|header| header.address().clone())
        .expect("Modified entry should be in chain");

    // Create Chain Entry

    let res : Result<Address, HolochainError> = zome_call_data.context
        .block_on(author_entry(
        &latest_entry,
        Some(chain_header_address.clone()),
            &zome_call_data.context.clone(),
        ));

    runtime.store_result(res)
}
