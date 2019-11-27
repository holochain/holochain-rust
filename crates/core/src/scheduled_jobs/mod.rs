pub mod state_dump;

use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use std::sync::Arc;

pub fn create_state_dump_callback(context: Arc<Context>) -> impl 'static + FnMut() + Sync + Send {
    move || {
        //log_debug!(context, "scheduled_jobs: tick");
        if context.state_dump_logging {
            state_dump::state_dump(context.clone());
        }
    }
}

pub fn create_state_pruning_callback(
    context: Arc<Context>,
) -> impl 'static + FnMut() + Sync + Send {
    move || {
        dispatch_action(context.action_channel(), ActionWrapper::new(Action::Prune));
    }
}
