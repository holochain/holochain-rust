use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    nucleus::{WasmApiFnCall, ZomeFnCall},
};
use std::sync::Arc;

pub fn trace_invoke_wasm_api_function(
    zome_fn_call: ZomeFnCall,
    wasm_api_fn_call: WasmApiFnCall,
    context: &Arc<Context>,
) {
    dispatch_action(
        context.action_channel(),
        ActionWrapper::new(Action::TraceInvokeWasmApiFunction((
            zome_fn_call,
            wasm_api_fn_call,
        ))),
    );
}
