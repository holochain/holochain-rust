use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};

use futures::{future::Future, task::Poll};
use holochain_core_types::error::HolochainError;
use holochain_persistence_api::cas::content::Address;
use std::{pin::Pin, sync::Arc,time::Instant};

/// Update Entry Action Creator
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(HolochainError).
pub fn update_entry(
    context: &Arc<Context>,
    old_address: Address,
    new_address: Address,
) -> UpdateEntryFuture {
    let action_wrapper = ActionWrapper::new(Action::UpdateEntry((old_address, new_address)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    UpdateEntryFuture {
        context: context.clone(),
        action: action_wrapper,
        running_time:Instant::now()
    }
}

/// RemoveEntryFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct UpdateEntryFuture {
    context: Arc<Context>,
    action: ActionWrapper,
    running_time:Instant
}

impl Future for UpdateEntryFuture {
    type Output = Result<Address, HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        self.context.future_trace.write().expect("Could not get future trace").capture(String::from("UpdateEntryFuture"),self.running_time.elapsed());
        
        if let Some(err) = self.context.action_channel_error("UpdateEntryFuture") {
            return Poll::Ready(Err(err));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        if let Some(state) = self.context.try_state() {
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
