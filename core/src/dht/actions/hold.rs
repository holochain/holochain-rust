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
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::Entry,
    error::HolochainError,
};
use std::{
    pin::{Pin, Unpin},
    sync::Arc,
};

pub async fn hold_entry<'a>(
    entry: &'a Entry,
    context: &'a Arc<Context>,
) -> Result<Address, HolochainError> {
    let action_wrapper = ActionWrapper::new(Action::Hold(entry.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    await!(HoldEntryFuture {
        context: context.clone(),
        address: entry.address(),
    })
}

pub struct HoldEntryFuture {
    context: Arc<Context>,
    address: Address,
}

impl Unpin for HoldEntryFuture {}

impl Future for HoldEntryFuture {
    type Output = Result<Address, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        if let Some(state) = self.context.state() {
            if state
                .dht()
                .content_storage()
                .read()
                .unwrap()
                .contains(&self.address)
                .unwrap()
            {
                Poll::Ready(Ok(self.address.clone()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
