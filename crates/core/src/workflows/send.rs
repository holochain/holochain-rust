use crate::{
    network::{actions::custom_send::custom_send, direct_message::CustomDirectMessage},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_json_api::json::JsonString;
use holochain_wasm_types::send::SendArgs;
use crate::wasm_engine::runtime::Runtime;
use holochain_wasm_types::WasmError;
use holochain_core_types::error::HolochainError;

/// ZomeApiFunction::Send function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: SendArgs
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_send(runtime: &mut Runtime, args: SendArgs) -> Result<JsonString, HolochainError> {
    let context = runtime.context().map_err(|e| WasmError::Zome(e.to_string()))?;
    let span = runtime
        .call_data()
        .ok()
        .map(|d| d.context.tracer.span("hdk invoke_send").start().into())
        .unwrap_or_else(|| ht::noop("hdk invoke_send no context"));
    let _trace_guard = ht::push_span(span);
    let call_data = runtime.call_data().map_err(|e| WasmError::Zome(e.to_string()))?;

    let message = CustomDirectMessage {
        payload: Ok(args.payload),
        zome: call_data.zome_name,
    };

    context
        .block_on(custom_send(args.to_agent, message, args.options.0, context.clone()))
        .map(|s| JsonString::from_json(&String::from(s)))
}
