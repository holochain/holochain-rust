extern crate futures;
use crate::{
    action::{Action, ActionWrapper},
    agent::state::ActionResponse,
    context::Context,
    instance::dispatch_action,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{cas::content::Address, entry::Entry, error::HolochainError};
use std::{
    pin::{Pin, Unpin},
    sync::{Arc},
};
//use core::mem::PinMut;

/// Publish Action Creator
/// This is the high-level publish function that wraps the whole publish process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an ActionResponse.
pub async fn publish_entry(
    address: Address,
    context: &Arc<Context>,
) -> Result<Address, HolochainError> {
    let action_wrapper = ActionWrapper::new(Action::Publish(address));
    dispatch_action(&context.action_channel, action_wrapper.clone());
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

impl Unpin for Publish {}

impl Future for Publish {
    type Output = Result<Address, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        match self
            .context
            .state()
            .unwrap()
            .network()
            .actions()
            .get(&self.action)
        {
            Some(ActionResponse::Publish(result)) => match result {
                Ok(address) => Poll::Ready(Ok(address.clone())),
                Err(error) => Poll::Ready(Err(error.clone())),
            },
            Some(_) => unreachable!(),
            None => Poll::Pending,
        }
    }
}
