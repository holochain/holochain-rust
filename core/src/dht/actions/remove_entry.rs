use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{cas::content::Address, error::HolochainError};
use std::{pin::Pin, sync::Arc};

/// Remove Entry Action Creator
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(HolochainError).
pub fn remove_entry(
    context: &Arc<Context>,
    deleted_address: Address,
    deletion_address: Address,
) -> Result<RemoveEntryFuture, HolochainError> {
    let action_wrapper = ActionWrapper::new(Action::RemoveEntry((
        deleted_address,
        deletion_address,
    )));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    Ok(RemoveEntryFuture {
        context: context.clone(),
        action: action_wrapper,
    })
}

/// RemoveEntryFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct RemoveEntryFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for RemoveEntryFuture {
    type Output = Result<(), HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        if let Some(state) = self.context.state() {
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
