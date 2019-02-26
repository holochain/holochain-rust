use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::author_entry::author_entry,
    network::entry_with_header::{EntryWithHeader,fetch_entry_with_header}
};

use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    link::{link_data::LinkData, LinkActionKind},
    cas::content::AddressableContent
};
use holochain_wasm_utils::api_serialization::link_entries::LinkEntriesArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::GetLinks function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: GetLinksArgs
/// Returns an HcApiReturnCode as I64
pub fn invoke_remove_link(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let zome_call_data = runtime.zome_call_data()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match LinkEntriesArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            zome_call_data.context.log(format!(
                "err/zome: invoke_link_entries failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let link = input.to_link();
    let link_remove = LinkData::from_link(&link, LinkActionKind::REMOVE);
    let entry = Entry::LinkRemove(link_remove);

    let mut entry_with_header_result = fetch_entry_with_header(&entry.address(),&zome_call_data.context.clone());
    if entry_with_header_result.is_err()
    {
        return ribosome_error_code!(Unspecified)
    }
    
    let entry_with_header = entry_with_header_result.unwrap();

    // Wait for future to be resolved
    let result: Result<(), HolochainError> = zome_call_data
        .context
        .block_on(author_entry(&entry_with_header,&zome_call_data.context))
        .map(|_| ());

    runtime.store_result(result)
}
