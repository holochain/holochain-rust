use crate::{
    network::{actions::custom_send::custom_send, direct_message::CustomDirectMessage},
};
use holochain_json_api::json::RawString;
use holochain_wasm_types::send::SendArgs;
use std::sync::Arc;
use crate::context::Context;
use crate::workflows::WorkflowResult;

/// ZomeApiFunction::Send function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: SendArgs
/// Returns an HcApiReturnCode as I64
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn send_workflow(context: Arc<Context>, args: &SendArgs) -> WorkflowResult<RawString> {
    let message = CustomDirectMessage {
        payload: Ok(args.payload.to_owned()),
        zome: args.zome.to_owned(),
    };

    Ok(RawString::from(custom_send(Arc::clone(&context), args.to_agent.to_owned(), message, args.options.0.to_owned()).await?))
}
