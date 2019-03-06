use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    scheduled_jobs::pending_validations::ValidatingWorkflow,
};
use holochain_core_types::cas::content::Address;
use std::sync::Arc;

pub fn remove_pending_validation(
    address: Address,
    workflow: ValidatingWorkflow,
    context: &Arc<Context>,
) {
    dispatch_action(
        context.action_channel(),
        ActionWrapper::new(Action::RemovePendingValidation((address, workflow))),
    );
}
