use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    dht::pending_validations::PendingValidation,
    instance::dispatch_action,
    
};
use futures::{future::Future, task::Poll};
use std::{
    pin::Pin,
    sync::Arc,
    time::{Duration, SystemTime},
};

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn dispatch_queue_holding_workflow(
    context: Arc<Context>,
    pending: PendingValidation,
    delay: Option<Duration>,
) {
    let delay_with_now = delay.map(|d| (SystemTime::now(), d));
    let action_wrapper =
        ActionWrapper::new(Action::QueueHoldingWorkflow((pending, delay_with_now)));
    dispatch_action(context.action_channel(), action_wrapper);
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn queue_holding_workflow(
    context: Arc<Context>,
    pending: PendingValidation,
    delay: Option<Duration>,
) {
    if !context
        .state()
        .expect("Can't queue holding workflow without state")
        .dht()
        .has_exact_queued_holding_workflow(&pending)
    {
        log_trace!(context, "Queueing holding workflow: {:?}", pending);
        dispatch_queue_holding_workflow(Arc::clone(&context), Arc::clone(&pending), delay);
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

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for QueueHoldingWorkflowFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();

        if let Some(state) = self.context.try_state() {
            if state.dht().has_exact_queued_holding_workflow(&self.pending) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
