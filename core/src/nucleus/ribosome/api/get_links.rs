use holochain_core_types::cas::content::Address;
use holochain_wasm_utils::api_serialization::get_links::{GetLinksArgs, GetLinksResult};
use nucleus::ribosome::Runtime;
use serde_json;
use std::collections::HashSet;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// ZomeApiFunction::GetLinks function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: GetLinksArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_get_links(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime.load_utf8_from_args(&args);
    let input: GetLinksArgs = match serde_json::from_str(&args_str) {
        Ok(input) => input,
        Err(_) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    let get_links_result = runtime.context.state().unwrap().dht().get_links(input.entry_address, input.tag);

    let json = serde_json::to_string(&GetLinksResult{
        ok: get_links_result.is_ok(),
        links: get_links_result
            .clone()
            .unwrap_or(HashSet::new())
            .iter()
            .map(|eav| eav.value())
            .collect::<Vec<Address>>(),
        error: get_links_result
            .map_err(|holochain_error| holochain_error.to_string())
            .err()
            .unwrap_or(String::from("")),
    }).expect("Could not serialize GetLinksResult");

    runtime.store_utf8(&json)
}
