use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use holochain_persistence_api::cas::content::Address;

use holochain_core_types::error::HolochainError;

use std::{pin::Pin, sync::Arc,time::{Instant,Duration}};

/// Remove Entry Action Creator
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(HolochainError).
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
        running_time:Instant::now()
    }
}

/// RemoveEntryFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct RemoveEntryFuture {
    context: Arc<Context>,
    action: ActionWrapper,
    running_time:Instant
}

impl Future for RemoveEntryFuture {
    type Output = Result<(), HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if self.running_time.elapsed() > Duration::from_secs(70)
        {
            panic!("future has been running for too long")
        }
        else
        {
            
        }
        self.context.future_trace.write().expect("Could not get future trace").start_capture(String::from("RemoveEntryFuture"));
        if let Some(err) = self.context.action_channel_error("RemoveEntryFuture") {
            return Poll::Ready(Err(err));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        if let Some(state) = self.context.try_state() {
        self.context.future_trace.write().expect("Could not get future trace").end_capture(String::from("RemoveEntryFuture"));
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
