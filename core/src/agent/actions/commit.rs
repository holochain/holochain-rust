extern crate futures;
use action::{Action, ActionWrapper};
use agent::state::ActionResponse;
use context::Context;
use futures::Future;
use hash_table::entry::Entry;
use instance::dispatch_action;
use std::sync::{mpsc::Sender, Arc};

pub fn commit_entry(
    entry: Entry,
    action_channel: &Sender<ActionWrapper>,
    context: &Arc<Context>,
) -> CommitFuture {
    let action_wrapper = ActionWrapper::new(Action::Commit(entry));
    dispatch_action(action_channel, action_wrapper.clone());
    CommitFuture {
        context: context.clone(),
        action: action_wrapper,
    }
}

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
