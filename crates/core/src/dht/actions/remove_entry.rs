use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use holochain_persistence_api::cas::content::Address;

use holochain_core_types::error::HolochainError;

use std::{pin::Pin, sync::Arc};

/// Remove Entry Action Creator
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(HolochainError).
#[cfg(not(target_arch = "wasm32"))]
#[flame]
pub fn remove_entry(
    context: &Arc<Context>,
    deleted_address: Address,
    deletion_address: Address,
) -> RemoveEntryFuture {
    let action_wrapper =
        ActionWrapper::new(Action::RemoveEntry((deleted_address, deletion_address)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    RemoveEntryFuture {
        context: context.clone(),
        action: action_wrapper,
    }
}

/// RemoveEntryFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct RemoveEntryFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for RemoveEntryFuture {
    type Output = Result<(), HolochainError>;


    #[cfg(not(target_arch = "wasm32"))]
    #[flame]
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {

        if let Some(err) = self.context.action_channel_error("RemoveEntryFuture") {
            return Poll::Ready(Err(err));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        if let Some(state) = self.context.try_state() {
            match state.dht().actions().get(&self.action) {
                Some(Ok(_)) => Poll::Ready(Ok(())),
                Some(Err(e)) => Poll::Ready(Err(e.clone())),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
