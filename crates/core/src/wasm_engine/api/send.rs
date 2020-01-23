use crate::{
    network::{actions::custom_send::custom_send, direct_message::CustomDirectMessage},
    wasm_engine::{api::ZomeApiResult, Runtime},
};
use holochain_json_api::json::JsonString;
use holochain_wasm_utils::api_serialization::send::SendArgs;

/// ZomeApiFunction::Send function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: SendArgs
/// Returns an HcApiReturnCode as I64
pub fn invoke_send(runtime: &mut Runtime, args: SendArgs) -> ZomeApiResult {
    let message = CustomDirectMessage {
        payload: Ok(args.payload),
        zome: runtime.call_data()?.zome_name,
    };

    let result = runtime
        .context()?
        .block_on(custom_send(
            args.to_agent,
            message,
            args.options.0,
            runtime.context()?,
        ))
        .map(|s| JsonString::from_json(&s));

    runtime.store_result(result)
}
