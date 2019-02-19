extern crate futures;
extern crate serde_json;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::entry_with_header::EntryWithHeader,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    error::HolochainError,
    crud_status::CrudStatus
};
use std::{pin::Pin, sync::Arc};

pub fn crud_status(
    entry_wh: EntryWithHeader,
    context: Arc<Context>,
    crud_status : CrudStatus
) -> CrudStatusFuture {
    let action_wrapper =
        ActionWrapper::new(Action::CrudStatus((entry_wh, crud_status)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    CrudStatusFuture {
        context: context.clone(),
        action: action_wrapper,
    }
}

pub struct CrudStatusFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for CrudStatusFuture {
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