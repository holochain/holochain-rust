pub mod get_entry;
pub mod init;
pub mod publish;
pub mod respond_get;

use crate::{
    action::{Action, ActionWrapper, NetworkReduceFn},
    context::Context,
    network::{
        reducers::{
            get_entry::reduce_get_entry, init::reduce_init,
            publish::reduce_publish, respond_get::reduce_respond_get,
        },
        state::NetworkState,
    },
};
use std::sync::Arc;

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<NetworkReduceFn> {
    match action_wrapper.action() {
        Action::InitNetwork(_) => Some(reduce_init),
        Action::Publish(_) => Some(reduce_publish),
        Action::GetEntry(_) => Some(reduce_get_entry),
        Action::RespondGet(_) => Some(reduce_respond_get),
        _ => None,
    }
}

pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<NetworkState>,
    action_wrapper: &ActionWrapper,
) -> Arc<NetworkState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: NetworkState = (*old_state).clone();
            f(context, &mut new_state, &action_wrapper);
            Arc::new(new_state)
        }
        None => old_state,
    }
}
