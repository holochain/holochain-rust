extern crate futures;
extern crate serde_json;
use action::{Action, ActionWrapper};
use context::Context;
use futures::{Async, Future};
use holochain_core_types::{error::HolochainError, links_entry::LinkEntry};
use instance::dispatch_action;
use std::sync::Arc;

/// ValidateEntry Action Creator
/// This is the high-level validate function that wraps the whole validation process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub fn add_link(link_entry: LinkEntry, context: &Arc<Context>) -> AddLinkFuture {
    let action_wrapper = ActionWrapper::new(Action::AddLink(link_entry.link().clone()));
    dispatch_action(&context.action_channel, action_wrapper.clone());

    AddLinkFuture {
        context: context.clone(),
        action: action_wrapper,
    }
}

/// ValidationFuture resolves to an Ok(ActionWrapper) or an Err(error_message:String).
/// Tracks the state for ValidationResults.
pub struct AddLinkFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

impl Future for AddLinkFuture {
    type Item = ();
    type Error = HolochainError;

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
            match state.dht().add_link_actions().get(&self.action) {
                Some(Ok(())) => Ok(futures::Async::Ready(())),
                Some(Err(e)) => Err(e.clone()),
                None => Ok(futures::Async::Pending),
            }
        } else {
            Ok(futures::Async::Pending)
        }
    }
}
