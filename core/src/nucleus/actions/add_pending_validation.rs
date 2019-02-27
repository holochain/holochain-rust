use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::entry_with_header::EntryWithHeader,
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
        ActionWrapper::new(Action::AddPendingValidation(Box::new((
            entry_with_header.to_owned(),
            dependencies.clone(),
        )))),
    );
}
