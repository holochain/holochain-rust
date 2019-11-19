use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    scheduled_jobs::pending_validations::PendingValidation,
};
use futures::{future::Future, task::Poll};
use std::{
    pin::Pin,
    sync::Arc,
    time::{Duration, SystemTime},
};

pub fn dispatch_queue_holding_workflow(
    pending: PendingValidation,
    delay: Option<Duration>,
    context: Arc<Context>,
) {
    let delay_with_now = delay.map(|d| (SystemTime::now(), d));
    let action_wrapper =
        ActionWrapper::new(Action::QueueHoldingWorkflow((pending, delay_with_now)));
    dispatch_action(context.action_channel(), action_wrapper.clone());
}

pub async fn queue_holding_workflow(
    pending: PendingValidation,
    delay: Option<Duration>,
    context: Arc<Context>,
) {
    if !context
        .state()
        .expect("Can't queue holding workflow without state")
        .dht()
        .has_queued_holding_workflow(&pending)
    {
        log_trace!(context, "Queueing holding workflow: {:?}", pending);
        dispatch_queue_holding_workflow(pending.clone(), delay, context.clone());
        QueueHoldingWorkflowFuture { context, pending }.await
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
}

impl Future for QueueHoldingWorkflowFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();

        if let Some(state) = self.context.try_state() {
            if state.dht().has_queued_holding_workflow(&self.pending) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
