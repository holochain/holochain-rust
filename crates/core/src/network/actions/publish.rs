use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::actions::NetworkActionResponse,NEW_RELIC_LICENSE_KEY
};
use futures::{future::Future, task::Poll};
use holochain_core_types::error::HcResult;
use holochain_persistence_api::cas::content::Address;
use std::{pin::Pin, sync::Arc};

/// Publish Action Creator
/// This is the high-level publish function that wraps the whole publish process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// Returns a future that resolves to an ActionResponse.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn publish(address: Address, context: &Arc<Context>) -> HcResult<Address> {
    let action_wrapper = ActionWrapper::new(Action::Publish(address));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    PublishFuture {
        context: context.clone(),
        action: action_wrapper,
    }
    .await
}

/// PublishFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct PublishFuture {
    context: Arc<Context>,
    action: ActionWrapper,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for PublishFuture {
    type Output = HcResult<Address>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("PublishFuture") {
            return Poll::Ready(Err(err));
        }

        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();

        if let Some(state) = self.context.try_state() {
            let state = state.network();
            if let Err(error) = state.initialized() {
                return Poll::Ready(Err(error));
            }

            match state.actions().get(&self.action) {
                Some(r) => match r.response() {
                    NetworkActionResponse::Publish(result) => {
                        dispatch_action(
                            self.context.action_channel(),
                            ActionWrapper::new(Action::ClearActionResponse(*self.action.id())),
                        );
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
