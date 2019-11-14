use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    scheduled_jobs::pending_validations::PendingValidation,
};

use std::sync::Arc;

pub fn pop_next_holding_workflow(pending: PendingValidation, context: Arc<Context>) {
    let action_wrapper = ActionWrapper::new(Action::PopNextHoldingWorkflow(pending));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}
