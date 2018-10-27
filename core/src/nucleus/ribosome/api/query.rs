use holochain_core_types::entry_type::EntryType;
use holochain_wasm_utils::api_serialization::{QueryArgs, QueryResult};
use nucleus::ribosome::Runtime;
use serde_json;
use std::str::FromStr;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// ZomeApiFunction::query function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: ?
/// Returns an HcApiReturnCode as I32
pub fn invoke_query(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let query = match QueryArgs::try_from(args_str) {
        Ok(input) => input,
        Err(_) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };
    // Get entry_type
    let maybe_entry_type = EntryType::from_str(&query.entry_type_name);
    if maybe_entry_type.is_err() {
        return ribosome_error_code!(UnknownEntryType);
    }
    let entry_type = maybe_entry_type.unwrap();
    // Perform query
    let agent = runtime.context.state().unwrap().agent();
    let top = agent
        .top_chain_header()
        .expect("Should have genesis entries.");
    let result = agent.chain().query(&Some(top), entry_type, query.limit);
    // Return result
    // let query_result = QueryResult { hashes: result };
    // let json = serde_json::to_string(&query_result).expect("Could not serialize QueryResult");
    runtime.store_as_json_string(result)
}
