extern crate futures;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::actions::ActionResponse,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{cas::content::Address, error::HcResult};
use std::{
    pin::{Pin, Unpin},
    sync::Arc,
};

/// Publish Action Creator
/// This is the high-level publish function that wraps the whole publish process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an ActionResponse.
pub async fn publish(address: Address, context: &Arc<Context>) -> HcResult<Address> {
    let action_wrapper = ActionWrapper::new(Action::Publish(address));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    await!(PublishFuture {
        context: context.clone(),
        action: action_wrapper,
    })
}

/// PublishFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct PublishFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Unpin for PublishFuture {}

impl Future for PublishFuture {
    type Output = HcResult<Address>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        let state = self.context.state().unwrap().network();
        if let Err(error) = state.initialized() {
            return Poll::Ready(Err(error));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        match state.actions().get(&self.action) {
            Some(ActionResponse::Publish(result)) => match result {
                Ok(address) => Poll::Ready(Ok(address.to_owned())),
                Err(error) => Poll::Ready(Err(error.clone())),
            },
            _ => Poll::Pending,
        }
    }
}
