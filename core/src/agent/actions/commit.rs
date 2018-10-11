extern crate futures;
use action::{Action, ActionWrapper};
use agent::state::ActionResponse;
use context::Context;
use futures::Future;
use holochain_core_types::{entry::Entry};
use instance::dispatch_action;
use std::sync::{mpsc::SyncSender, Arc};

/// Commit Action Creator
/// This is the high-level commit function that wraps the whole commit process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an ActionResponse.
pub fn commit_entry(
    entry: Entry,
    action_channel: &SyncSender<ActionWrapper>,
    context: &Arc<Context>,
) -> CommitFuture {
    let action_wrapper = ActionWrapper::new(Action::Commit(entry));
    dispatch_action(action_channel, action_wrapper.clone());
    CommitFuture {
        context: context.clone(),
        action: action_wrapper,
    }
}

/// CommitFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct CommitFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for CommitFuture {
    type Item = ActionResponse;
    type Error = String;

    fn poll(
        &mut self,
        cx: &mut futures::task::Context<'_>,
    ) -> Result<futures::Async<ActionResponse>, Self::Error> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().wake();
        match self
            .context
            .state()
            .unwrap()
            .agent()
            .actions()
            .get(&self.action)
        {
            Some(response) => Ok(futures::Async::Ready(response.clone())),
            None => Ok(futures::Async::Pending),
        }
    }
}
