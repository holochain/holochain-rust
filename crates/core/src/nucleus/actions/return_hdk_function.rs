use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    nucleus::{HdkFnCall, HdkFnCallResult, ZomeFnCall},
};
use std::sync::Arc;

pub fn return_hdk_function(
    zome_fn_call: ZomeFnCall,
    hdk_fn_call: HdkFnCall,
    hdk_fn_call_result: HdkFnCallResult,
    context: &Arc<Context>,
) {
    dispatch_action(
        context.action_channel(),
        ActionWrapper::new(Action::ReturnHdkFunction((
            zome_fn_call,
            hdk_fn_call,
            hdk_fn_call_result,
        ))),
    );
}
