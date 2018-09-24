extern crate futures;
use action::{Action, ActionWrapper};
use context::Context;
use futures::{Async, Future};
use hash_table::entry::Entry;
use instance::dispatch_action;
use std::sync::{mpsc::Sender, Arc};

/// ValidateEntry Action Creator
/// This is the high-level validate function that wraps the whole validation process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub fn validate_entry(
    entry: Entry,
    action_channel: &Sender<ActionWrapper>,
    context: &Arc<Context>,
) -> ValidationFuture {
    let action_wrapper = ActionWrapper::new(Action::ValidateEntry(entry));
    dispatch_action(action_channel, action_wrapper.clone());
    ValidationFuture {
        context: context.clone(),
        action: action_wrapper,
    }
}

/// ValidationFuture resolves to an Ok(ActionWrapper) or an Err(error_message:String).
/// Tracks the state for ValidationResults.
pub struct ValidationFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for ValidationFuture {
    type Item = ActionWrapper;
    type Error = String;

    fn poll(
        &mut self,
        cx: &mut futures::task::Context<'_>,
    ) -> Result<Async<Self::Item>, Self::Error> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().wake();
        if let Some(state) = self.context.state() {
            match state.nucleus().validation_result(&self.action) {
                Some(Ok(())) => Ok(futures::Async::Ready(self.action.clone())),
                Some(Err(e)) => Err(e),
                None => Ok(futures::Async::Pending),
            }
        } else {
            Ok(futures::Async::Pending)
        }
    }
}
