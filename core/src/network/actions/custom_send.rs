extern crate futures;
use crate::{
    action::{Action, ActionWrapper, DirectMessageData},
    context::Context,
    instance::dispatch_action,
    network::direct_message::{CustomDirectMessage, DirectMessage},
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{
    cas::content::Address, error::HcResult,
};
use std::{
    pin::{Pin, Unpin},
    sync::Arc,
};
use snowflake::ProcessUniqueId;

/// GetValidationPackage Action Creator
/// This triggers the network module to retrieve the validation package for the
/// entry given by the header.
///
/// Returns a future that resolves to Option<ValidationPackage> (or HolochainError).
/// If that is None this means that we couldn't get a validation package from the source.
pub async fn custom_send(
    to_agent: Address,
    custom_direct_message: CustomDirectMessage,
    context: &Arc<Context>,
) -> HcResult<CustomDirectMessage> {
    let id = ProcessUniqueId::new().to_string();
    let direct_message = DirectMessage::Custom(custom_direct_message);
    let direct_message_data = DirectMessageData {
        address: to_agent,
        message: direct_message,
        msg_id: id.clone(),
        is_response: false,
    };
    let action_wrapper = ActionWrapper::new(Action::SendDirectMessage(direct_message_data));
    dispatch_action(&context.action_channel, action_wrapper);

    await!(SendResponseFuture {
        context: context.clone(),
        id,
    })
}

/// GetValidationPackageFuture resolves to an Option<ValidationPackage>
/// which would be None if the source responded with None, indicating that it
/// is not the source.
pub struct SendResponseFuture {
    context: Arc<Context>,
    id: String,
}

impl Unpin for SendResponseFuture {}

impl Future for SendResponseFuture {
    type Output = HcResult<CustomDirectMessage>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        let state = self.context.state().unwrap().network();
        if let Err(error) = state.initialized() {
            return Poll::Ready(Err(error));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        match state.custom_direct_message_replys.get(&self.id) {
            Some(result) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
