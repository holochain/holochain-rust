use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use holochain_core_types::{entry::Entry, error::HolochainError};
use std::{pin::Pin, sync::Arc, time::Instant};

/// RemoveLink Action Creator
/// This action creator dispatches an RemoveLink action which is consumed by the DHT reducer.
/// Note that this function does not include any validation checks for the link.
/// The DHT reducer does make sure that it only removes links to a base that it has in its
/// local storage and will return an error that the RemoveLinkFuture resolves to
/// if that is not the case.
///
/// Returns a future that resolves to an Ok(()) or an Err(HolochainError).
pub fn remove_link(entry: &Entry, context: &Arc<Context>) -> RemoveLinkFuture {
    let action_wrapper = ActionWrapper::new(Action::RemoveLink(entry.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    RemoveLinkFuture {
        context: context.clone(),
        action: action_wrapper,
        running_time: Instant::now(),
    }
}

pub struct RemoveLinkFuture {
    context: Arc<Context>,
    action: ActionWrapper,
    running_time: Instant,
}

impl Unpin for RemoveLinkFuture {}

impl Future for RemoveLinkFuture {
    type Output = Result<(), HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        self.context
            .future_trace
            .write()
            .expect("Could not get future trace")
            .capture(
                String::from("RemoveLinkFuture"),
                self.running_time.elapsed(),
            );
        if let Some(err) = self.context.action_channel_error("RemoveLinkFuture") {
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
