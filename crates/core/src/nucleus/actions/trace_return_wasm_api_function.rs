use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    nucleus::{WasmApiFnCall, WasmApiFnCallResult, ZomeFnCall},
};
use std::sync::Arc;

pub fn trace_return_wasm_api_function(
    zome_fn_call: ZomeFnCall,
    wasm_api_fn_call: WasmApiFnCall,
    wasm_api_fn_call_result: WasmApiFnCallResult,
    context: &Arc<Context>,
) {
    dispatch_action(
        context.action_channel(),
        ActionWrapper::new(Action::TraceReturnWasmApiFunction((
            zome_fn_call,
            wasm_api_fn_call,
            wasm_api_fn_call_result,
        ))),
    );
}
