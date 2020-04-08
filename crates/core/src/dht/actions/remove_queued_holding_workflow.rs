use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::pending_validations::PendingValidation,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc};

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn remove_queued_holding_workflow(pending: PendingValidation, context: Arc<Context>) {
    let action_wrapper = ActionWrapper::new(Action::RemoveQueuedHoldingWorkflow(pending.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    let id = ProcessUniqueId::new();
    RemoveQueuedHoldingWorkflowFuture {
        context,
        pending,
        id,
    }
    .await
}

pub struct RemoveQueuedHoldingWorkflowFuture {
    context: Arc<Context>,
    pending: PendingValidation,
    id: ProcessUniqueId,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for RemoveQueuedHoldingWorkflowFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        self.context
            .register_waker(self.id.clone(), cx.waker().clone());

        if let Some(state) = self.context.try_state() {
            if state.dht().has_exact_queued_holding_workflow(&self.pending) {
                Poll::Pending
            } else {
                self.context.unregister_waker(self.id.clone());
                Poll::Ready(())
            }
        } else {
            Poll::Pending
        }
    }
}
