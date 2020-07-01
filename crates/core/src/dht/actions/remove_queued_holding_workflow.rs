use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::pending_validations::PendingValidation,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc, time::Duration};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum HoldingWorkflowQueueing {
    Processing,
    Waiting(Duration),
    Done,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn remove_queued_holding_workflow(
    state: HoldingWorkflowQueueing,
    pending: PendingValidation,
    context: Arc<Context>,
) {
    let action_wrapper = ActionWrapper::new(Action::RemoveQueuedHoldingWorkflow((
        state.clone(),
        pending.clone(),
    )));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    let id = ProcessUniqueId::new();
    RemoveQueuedHoldingWorkflowFuture {
        context,
        pending,
        state,
        id,
    }
    .await
}

pub struct RemoveQueuedHoldingWorkflowFuture {
    context: Arc<Context>,
    pending: PendingValidation,
    state: HoldingWorkflowQueueing,
    id: ProcessUniqueId,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for RemoveQueuedHoldingWorkflowFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        self.context
            .register_waker(self.id.clone(), cx.waker().clone());

        if let Some(state) = self.context.try_state() {
            let store = state.dht();
            match self.state {
                HoldingWorkflowQueueing::Processing => {
                    if store.has_exact_queued_holding_workflow(&self.pending)
                        || !store.has_exact_in_process_holding_workflow(&self.pending)
                    {
                        Poll::Pending
                    } else {
                        self.context.unregister_waker(self.id.clone());
                        Poll::Ready(())
                    }
                }
                HoldingWorkflowQueueing::Waiting(_) => {
                    if !store.has_exact_queued_holding_workflow(&self.pending)
                        || store.has_exact_in_process_holding_workflow(&self.pending)
                    {
                        Poll::Pending
                    } else {
                        self.context.unregister_waker(self.id.clone());
                        Poll::Ready(())
                    }
                }
                HoldingWorkflowQueueing::Done => {
                    if store.has_exact_in_process_holding_workflow(&self.pending) {
                        Poll::Pending
                    } else {
                        self.context.unregister_waker(self.id.clone());
                        Poll::Ready(())
                    }
                }
            }
        } else {
            Poll::Pending
        }
    }
}
