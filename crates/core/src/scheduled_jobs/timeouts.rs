use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use std::sync::Arc;

pub fn check_network_processes_for_timeouts(context: Arc<Context>) {
    let state = context.state().expect("Couldn't get state in timeout job");
    for (key, (time, duration)) in state.network().query_timeouts.iter() {
        if let Ok(elapsed) = time.elapsed() {
            if elapsed > *duration {
                dispatch_action(
                    context.action_channel(),
                    ActionWrapper::new(Action::QueryTimeout(key.clone())),
                );
            }
        }
    }

    for (key, (time, duration)) in state.network().direct_message_timeouts.iter() {
        if let Ok(elapsed) = time.elapsed() {
            if elapsed > *duration {
                dispatch_action(
                    context.action_channel(),
                    ActionWrapper::new(Action::SendDirectMessageTimeout(key.clone())),
                );
            }
        }
    }
}
