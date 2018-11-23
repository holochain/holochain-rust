use holochain_core_types::entry::entry_type::EntryType;
use holochain_wasm_utils::api_serialization::QueryArgs;
use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use std::{convert::TryFrom, str::FromStr};
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::query function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: ?
/// Returns an HcApiReturnCode as I32
pub fn invoke_query(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let query = match QueryArgs::try_from(args_str) {
        Ok(input) => input,
        Err(..) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    // Get entry_type
    let entry_type = match EntryType::from_str(&query.entry_type_name) {
        Ok(inner) => inner,
        Err(..) => return ribosome_error_code!(UnknownEntryType),
    };

    // Perform query
    let agent = runtime.context.state().unwrap().agent();
    let top = agent
        .top_chain_header()
        .expect("Should have genesis entries.");

    runtime.store_result(Ok(agent.chain().query(
        &Some(top),
        &entry_type,
        query.start,
        query.limit,
    )))
}
