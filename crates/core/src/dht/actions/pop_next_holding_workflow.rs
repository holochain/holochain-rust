use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};

use std::sync::Arc;

pub fn pop_next_holding_workflow(context: Arc<Context>) {
    let action_wrapper = ActionWrapper::new(Action::PopNextHoldingWorkflow);
    dispatch_action(context.action_channel(), action_wrapper.clone());
}
