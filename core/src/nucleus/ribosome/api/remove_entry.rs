use crate::{
    nucleus::{ribosome::{api::ZomeApiResult, Runtime},
    actions::{build_validation_package::*, validate::*}},
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},
    network::entry_with_header::{EntryWithHeader,fetch_entry_with_header}
};
use futures::future::{self, TryFutureExt};
use holochain_core_types::{
    cas::content::{Address,AddressableContent},
    entry::{deletion_entry::DeletionEntry, Entry},
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
};
use holochain_wasm_utils::api_serialization::get_entry::*;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::RemoveEntry function code
/// args: [0] encoded MemoryAllocation
/// Expected Address argument
/// Stores/returns a RibosomeEncodedValue
pub fn invoke_remove_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    println!("invoked");
    let zome_call_data = runtime.zome_call_data()?;
    println!("zome call adat");
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let try_address = Address::try_from(args_str.clone());
    println!("try address{:?}",try_address.clone());
    // Exit on error
    if try_address.is_err() {
        zome_call_data.context.log(format!(
            "err/zome: invoke_remove_entry failed to deserialize Address: {:?}",
            args_str
        ));
        return ribosome_error_code!(ArgumentDeserializationFailed);
    }
    let deleted_entry_address = try_address.unwrap();

    // Get Current entry's latest version
    let get_args = GetEntryArgs {
        address: deleted_entry_address,
        options: Default::default(),
    };
    let maybe_entry_result = zome_call_data.context.block_on(get_entry_result_workflow(
        &zome_call_data.context,
        &get_args,
    ));
    println!("maybe entry result {:?}",maybe_entry_result.clone());
    if let Err(_err) = maybe_entry_result {
        return ribosome_error_code!(Unspecified);
    }
    let entry_result = maybe_entry_result.unwrap();
    if !entry_result.found() {
        return ribosome_error_code!(Unspecified);
    }
    let deleted_entry_address = entry_result.latest().unwrap().address();

    let mut entry_with_header_result = fetch_entry_with_header(&deleted_entry_address,&zome_call_data.context.clone());
    println!("fetch result {:?}",entry_with_header_result.clone());
    if entry_with_header_result.is_err()
    {
        return ribosome_error_code!(Unspecified)
    }

    let entry_with_header = entry_with_header_result.unwrap();

    // Create deletion entry
    let deletion_entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address.clone()));
    let deletion_entry_with_header = EntryWithHeader{entry:deletion_entry.clone(),header:entry_with_header.header.clone()};

    let res: Result<(), HolochainError> = zome_call_data
        .context
        .block_on(author_entry(
            &deletion_entry_with_header,
            &zome_call_data.context.clone(),
        ))
        .map(|_| ());

    runtime.store_result(res)
}
