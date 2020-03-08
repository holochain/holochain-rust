use crate::{
    action::{Action, ActionWrapper, DirectMessageData},
    context::Context,
    instance::dispatch_action,
    network::direct_message::{CustomDirectMessage, DirectMessage},
};
use futures::{future::Future, task::Poll};
use holochain_core_types::{error::HolochainError, time::Timeout};
use holochain_persistence_api::cas::content::Address;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc, time::SystemTime};

/// SendDirectMessage Action Creator for custom (=app) messages
/// This triggers the network module to open a synchronous node-to-node connection
/// by sending the given CustomDirectMessage and preparing to receive a response.
//#[autotrace]
//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn custom_send(
    to_agent: Address,
    custom_direct_message: CustomDirectMessage,
    timeout: Timeout,
    context: Arc<Context>,
) -> Result<String, HolochainError> {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
    let id = format!("{}-{}", ProcessUniqueId::new().to_string(), rand_string);
    let direct_message = DirectMessage::Custom(custom_direct_message);
    let direct_message_data = DirectMessageData {
        address: to_agent,
        message: direct_message,
        msg_id: id.clone(),
        is_response: false,
    };
    let action_wrapper = ActionWrapper::new(Action::SendDirectMessage((
        direct_message_data,
        Some((SystemTime::now(), timeout.into())),
    )));
    dispatch_action(context.action_channel(), action_wrapper);

    SendResponseFuture {
        context: context.clone(),
        id,
    }
    .await
}

/// SendResponseFuture waits for a result to show up in NetworkState::custom_direct_message_replys
pub struct SendResponseFuture {
    context: Arc<Context>,
    id: String,
}

//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl Future for SendResponseFuture {
    type Output = Result<String, HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("SendResponseFuture") {
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
                return Poll::Ready(Err(HolochainError::ErrorGeneric(error.to_string())));
            }
            match state.custom_direct_message_replys.get(&self.id) {
                Some(result) => {
                    dispatch_action(
                        self.context.action_channel(),
                        ActionWrapper::new(Action::ClearCustomSendResponse(self.id.clone())),
                    );
                    Poll::Ready(result.clone())
                }
                _ => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
