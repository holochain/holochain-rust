use crate::{
    network::{actions::custom_send::custom_send, direct_message::CustomDirectMessage},
    wasm_engine::{api::ZomeApiResult},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_json_api::json::JsonString;
use std::sync::Arc;
use crate::context::Context;
use holochain_wasm_utils::api_serialization::send::SendArgs;

/// ZomeApiFunction::Send function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: SendArgs
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_send(context: Arc<Context>, args: SendArgs) -> ZomeApiResult {
    let span = context
        .call_data()
        .ok()
        .map(|d| d.context.tracer.span("hdk invoke_send").start().into())
        .unwrap_or_else(|| ht::noop("hdk invoke_send no context"));
    let _trace_guard = ht::push_span(span);
    let call_data = context.call_data()?;

    let message = CustomDirectMessage {
        payload: Ok(args.payload),
        zome: context.call_data()?.zome_name,
    };

    context
        .block_on(custom_send(
            args.to_agent,
            message,
            args.options.0,
            context,
        ))
        .map(|s| JsonString::from_json(&s))
}
