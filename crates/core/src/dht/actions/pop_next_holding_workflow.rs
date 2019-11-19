use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    scheduled_jobs::pending_validations::PendingValidation,
};
use futures::{future::Future, task::Poll};
use std::{pin::Pin, sync::Arc};

pub async fn pop_next_holding_workflow(pending: PendingValidation, context: Arc<Context>) {
    let action_wrapper = ActionWrapper::new(Action::PopNextHoldingWorkflow(pending.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    PopNextHoldingWorkflowFuture { context, pending }.await
}

pub struct PopNextHoldingWorkflowFuture {
    context: Arc<Context>,
    pending: PendingValidation,
}

impl Future for PopNextHoldingWorkflowFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();

        if let Some(state) = self.context.try_state() {
            match state.dht().next_queued_holding_workflow() {
                Some(head) => {
                    if *head.0 == self.pending {
                        Poll::Pending
                    } else {
                        Poll::Ready(())
                    }
                }
                None => Poll::Ready(()),
            }
        } else {
            Poll::Pending
        }
    }
}
