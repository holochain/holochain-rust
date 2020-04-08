use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::pending_validations::PendingValidation,
    instance::dispatch_action,
};
use futures::{future::Future, task::Poll};
use snowflake::ProcessUniqueId;
use std::{
    pin::Pin,
    sync::Arc,
    time::{Duration, SystemTime},
};

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn dispatch_queue_holding_workflow(
    pending: PendingValidation,
    delay: Option<Duration>,
    context: Arc<Context>,
) {
    let delay_with_now = delay.map(|d| (SystemTime::now(), d));
    let action_wrapper =
        ActionWrapper::new(Action::QueueHoldingWorkflow((pending, delay_with_now)));
    dispatch_action(context.action_channel(), action_wrapper);
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn queue_holding_workflow(
    pending: PendingValidation,
    delay: Option<Duration>,
    context: Arc<Context>,
) {
    if !context
        .state()
        .expect("Can't queue holding workflow without state")
        .dht()
        .has_exact_queued_holding_workflow(&pending)
    {
        log_trace!(context, "Queueing holding workflow: {:?}", pending);
        dispatch_queue_holding_workflow(pending.clone(), delay, context.clone());
        let id = ProcessUniqueId::new();
        QueueHoldingWorkflowFuture {
            context,
            pending,
            id,
        }
        .await
    } else {
        log_trace!(
            context,
            "Not queueing holding workflow since it is queued already: {:?}",
            pending
        );
    }
}

pub struct QueueHoldingWorkflowFuture {
    context: Arc<Context>,
    pending: PendingValidation,
    id: ProcessUniqueId,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for QueueHoldingWorkflowFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        self.context
            .register_waker(self.id.clone().into(), cx.waker().clone());

        if let Some(state) = self.context.try_state() {
            if state.dht().has_exact_queued_holding_workflow(&self.pending) {
                self.context.unregister_waker(self.id.clone().into());
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
