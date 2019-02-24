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

/// Crud Link Action Creator
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(HolochainError).
pub fn crud_link(
    context: &Arc<Context>,
    address: Address,
    crud_link: Address,
) -> Result<CrudLinkFuture, HolochainError> {
    let action_wrapper = ActionWrapper::new(Action::CrudLink((address, crud_link)));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    Ok(CrudLinkFuture {
        context: context.clone(),
        action: action_wrapper,
    })
}

/// CrudLinkFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct CrudLinkFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for CrudLinkFuture {
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
