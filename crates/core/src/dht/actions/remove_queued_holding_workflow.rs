use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::pending_validations::PendingValidation,
    instance::dispatch_action,
    
};
use futures::{future::Future, task::Poll};
use std::{pin::Pin, sync::Arc};

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn remove_queued_holding_workflow(context: Arc<Context>, pending: PendingValidation) {
    let action_wrapper = ActionWrapper::new(Action::RemoveQueuedHoldingWorkflow(pending.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    RemoveQueuedHoldingWorkflowFuture { context, pending }.await
}

pub struct RemoveQueuedHoldingWorkflowFuture {
    context: Arc<Context>,
    pending: PendingValidation,
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for RemoveQueuedHoldingWorkflowFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();

        if let Some(state) = self.context.try_state() {
            if state.dht().has_exact_queued_holding_workflow(&self.pending) {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        } else {
            Poll::Pending
        }
    }
}
