use crate::{
    action::{Action, ActionWrapper},
    agent::state::AgentActionResponse,
    context::Context,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use holochain_core_types::{entry::Entry, error::HolochainError};
use holochain_persistence_api::cas::content::Address;
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc};

/// Commit Action Creator
/// This is the high-level commit function that wraps the whole commit process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an ActionResponse.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
    let id = ProcessUniqueId::new();
    CommitFuture {
        context: context.clone(),
        action: action_wrapper,
        id,
    }
    .await
}

/// CommitFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct CommitFuture {
    context: Arc<Context>,
    action: ActionWrapper,
    id: ProcessUniqueId,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for CommitFuture {
    type Output = Result<Address, HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("CommitFuture") {
            return Poll::Ready(Err(err));
        }
        self.context
            .register_waker(self.id.clone(), cx.waker().clone());
        if let Some(state) = self.context.try_state() {
            match state.agent().actions().get(&self.action) {
                Some(r) => match r.response() {
                    AgentActionResponse::Commit(result) => {
                        dispatch_action(
                            self.context.action_channel(),
                            ActionWrapper::new(Action::ClearActionResponse(
                                self.action.id().to_string(),
                            )),
                        );
                        self.context.unregister_waker(self.id.clone());
                        Poll::Ready(result.clone())
                    }
                    _ => unreachable!(),
                },
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
