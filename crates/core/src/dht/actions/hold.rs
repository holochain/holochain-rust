use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::entry_with_header::EntryWithHeader,
};
use futures::{future::Future, task::Poll};
use holochain_core_types::error::HolochainError;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use std::{pin::Pin, sync::Arc};

pub async fn hold_entry(
    entry_wh: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<Address, HolochainError> {
    let address = entry_wh.entry.address();
    let action_wrapper = ActionWrapper::new(Action::Hold(entry_wh.to_owned()));
    dispatch_action(context.action_channel(), action_wrapper.clone());
    HoldEntryFuture { context, address }.await
}

pub struct HoldEntryFuture {
    context: Arc<Context>,
    address: Address,
}

impl Future for HoldEntryFuture {
    type Output = Result<Address, HolochainError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        self.context.future_trace.write().expect("Could not get future trace").capture();
        if let Some(err) = self.context.action_channel_error("HoldEntryFuture") {
            return Poll::Ready(Err(err));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        self.context.future_trace.write().expect("Could not get future trace").record_diagnostic(String::from("HoldEntryFuture"));
        if let Some(state) = self.context.try_state() {
            if state
                .dht()
                .content_storage()
                .read()
                .unwrap()
                .contains(&self.address)
                .unwrap()
            {
                Poll::Ready(Ok(self.address.clone()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
