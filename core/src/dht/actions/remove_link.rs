extern crate futures;
extern crate serde_json;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{error::HolochainError, link::Link};
use std::{pin::Pin, sync::Arc};

/// RemoveLink Action Creator
/// This action creator dispatches an RemoveLink action which is consumed by the DHT reducer.
/// Note that this function does not include any validation checks for the link.
/// The DHT reducer does make sure that it only removes links to a base that it has in its
/// local storage and will return an error that the RemoveLinkFuture resolves to
/// if that is not the case.
///
/// Returns a future that resolves to an Ok(()) or an Err(HolochainError).
pub fn remove_link(link: &Link, context: &Arc<Context>) -> RemoveLinkFuture {
    let action_wrapper = ActionWrapper::new(Action::RemoveLink(link.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    RemoveLinkFuture {
        context: context.clone(),
        action: action_wrapper,
    }
}

pub struct RemoveLinkFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Unpin for RemoveLinkFuture {}

impl Future for RemoveLinkFuture {
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
