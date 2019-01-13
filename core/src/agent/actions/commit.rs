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
use std::{pin::Pin, sync::Arc};
//use core::mem::PinMut;

/// Commit Action Creator
/// This is the high-level commit function that wraps the whole commit process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an ActionResponse.
pub async fn commit_entry(
    entry: Entry,
    maybe_crud_link: Option<Address>,
    context: &Arc<Context>,
) -> Result<Address, HolochainError> {
    let action_wrapper = ActionWrapper::new(Action::Commit((entry, maybe_crud_link)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    await!(CommitFuture {
        context: context.clone(),
        action: action_wrapper,
    })
}

/// CommitFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct CommitFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for CommitFuture {
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
            .agent()
            .actions()
            .get(&self.action)
        {
            Some(ActionResponse::Commit(result)) => match result {
                Ok(address) => Poll::Ready(Ok(address.clone())),
                Err(error) => Poll::Ready(Err(error.clone())),
            },
            Some(_) => unreachable!(),
            None => Poll::Pending,
        }
    }
}
