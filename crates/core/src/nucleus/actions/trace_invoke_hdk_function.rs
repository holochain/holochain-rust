use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    nucleus::{HdkFnCall, ZomeFnCall},
};
use std::sync::Arc;

pub fn trace_invoke_hdk_function(
    zome_fn_call: ZomeFnCall,
    hdk_fn_call: HdkFnCall,
    context: &Arc<Context>,
) {
    dispatch_action(
        context.action_channel(),
        ActionWrapper::new(Action::TraceInvokeHdkFunction((zome_fn_call, hdk_fn_call))),
    );
}
