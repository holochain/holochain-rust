use crate::{
    action::{Action, ActionWrapper},
    agent::state::ActionResponse,
    context::Context,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use holochain_core_types::{entry::Entry, error::HolochainError};
use holochain_persistence_api::cas::content::Address;
use std::{pin::Pin, sync::Arc};

/// Commit Action Creator
/// This is the high-level commit function that wraps the whole commit process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an ActionResponse.
#[cfg(not(target_arch = "wasm32"))]
#[flame]
pub async fn commit_entry(
    entry: Entry,
    maybe_link_update_delete: Option<Address>,
    context: &Arc<Context>,
) -> Result<Address, HolochainError> {
    let action_wrapper = ActionWrapper::new(Action::Commit((
        entry.clone(),
        maybe_link_update_delete,
        vec![],
    )));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    CommitFuture {
        context: context.clone(),
        action: action_wrapper,
    }
    .await
}

/// CommitFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct CommitFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for CommitFuture {
    type Output = Result<Address, HolochainError>;

    #[cfg(not(target_arch = "wasm32"))]
    #[flame]
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("CommitFuture") {
            return Poll::Ready(Err(err));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        if let Some(state) = self.context.try_state() {
            match state.agent().actions().get(&self.action) {
                Some(ActionResponse::Commit(result)) => match result {
                    Ok(address) => Poll::Ready(Ok(address.clone())),
                    Err(error) => Poll::Ready(Err(error.clone())),
                },
                Some(_) => unreachable!(),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
