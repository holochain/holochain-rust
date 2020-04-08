use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::actions::NetworkActionResponse,
};
use futures::{future::Future, task::Poll};
use holochain_core_types::error::HcResult;
use holochain_persistence_api::cas::content::Address;
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc};

/// Publish Header Entry Action Creator
/// Returns a future that resolves to an ActionResponse.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn publish_header_entry(address: Address, context: &Arc<Context>) -> HcResult<Address> {
    let action_wrapper = ActionWrapper::new(Action::PublishHeaderEntry(address));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    let id = ProcessUniqueId::new();
    PublishHeaderEntryFuture {
        context: context.clone(),
        action: action_wrapper,
        id,
    }
    .await
}

/// PublishFuture resolves to ActionResponse
/// Tracks the state for a response to its ActionWrapper
pub struct PublishHeaderEntryFuture {
    context: Arc<Context>,
    action: ActionWrapper,
    id: ProcessUniqueId,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for PublishHeaderEntryFuture {
    type Output = HcResult<Address>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self
            .context
            .action_channel_error("PublishHeaderEntryFuture")
        {
            return Poll::Ready(Err(err));
        }

        self.context
            .register_waker(self.id.clone(), cx.waker().clone());

        if let Some(state) = self.context.try_state() {
            let state = state.network();
            if let Err(error) = state.initialized() {
                return Poll::Ready(Err(error));
            }
            match state.actions().get(&self.action) {
                Some(r) => match r.response() {
                    NetworkActionResponse::PublishHeaderEntry(result) => {
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
                _ => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
