use crate::{
    network::{actions::custom_send::custom_send, direct_message::CustomDirectMessage},
    wasm_engine::{api::ZomeApiResult, Runtime},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_json_api::json::JsonString;
use holochain_wasm_utils::api_serialization::send::SendArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Send function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: SendArgs
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_send(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let span = runtime
        .call_data()
        .ok()
        .map(|d| d.context.tracer.span("hdk invoke_send").start().into())
        .unwrap_or_else(|| ht::noop("hdk invoke_send no context"));
    let _trace_guard = ht::push_span(span);
    let call_data = runtime.call_data()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let args = match SendArgs::try_from(args_str) {
        Ok(input) => input,
        Err(..) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    let message = CustomDirectMessage {
        payload: Ok(args.payload),
        zome: call_data.zome_name.clone(),
    };

    let result = call_data
        .context
        .block_on(custom_send(
            args.to_agent,
            message,
            args.options.0,
            call_data.context.clone(),
        ))
        .map(|s| JsonString::from_json(&s));

    runtime.store_result(result)
}
