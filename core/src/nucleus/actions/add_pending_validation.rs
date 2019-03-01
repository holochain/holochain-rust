use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::entry_with_header::EntryWithHeader,
    scheduled_jobs::pending_validations::PendingValidationStruct,
};
use holochain_core_types::cas::content::Address;
use std::sync::Arc;

pub fn add_pending_validation(
    entry_with_header: EntryWithHeader,
    dependencies: Vec<Address>,
    context: &Arc<Context>,
) {
    dispatch_action(
        context.action_channel(),
        ActionWrapper::new(Action::AddPendingValidation(Arc::new(
            PendingValidationStruct {
                entry_with_header,
                dependencies,
            },
        ))),
    );
}
