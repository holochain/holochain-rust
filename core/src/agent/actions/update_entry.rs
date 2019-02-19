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
use std::{
    pin::Pin,
    sync::{mpsc::SyncSender, Arc},
};

/// Update Entry Action Creator
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(HolochainError).
pub fn update_entry(
    context: &Arc<Context>,
    action_channel: &SyncSender<ActionWrapper>,
    old_address: Address,
    new_address: Address,
) -> UpdateEntryFuture {
    let action_wrapper = ActionWrapper::new(Action::UpdateEntry((old_address, new_address)));
    dispatch_action(action_channel, action_wrapper.clone());
    UpdateEntryFuture {
        context: context.clone(),
        action: action_wrapper,
    }
}

/// RemoveEntryFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct UpdateEntryFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for UpdateEntryFuture {
    type Output = Result<Address, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        if let Some(state) = self.context.state() {
            match state.dht().actions().get(&self.action) {
                Some(Ok(address)) => Poll::Ready(Ok(address.clone())),
                Some(Err(e)) => Poll::Ready(Err(e.clone())),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
