extern crate futures;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::actions::ActionResponse,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{
    cas::content::Address,
    error::{HcResult, HolochainError},
};
use snowflake;
use std::{
    pin::{Pin, Unpin},
    sync::Arc,
};

/// GetEntry Action Creator
/// This is the network version of get_entry that makes the network module start
/// a look-up process.
///
/// Returns a future that resolves to an ActionResponse.
pub async fn get_entry(address: Address, context: &Arc<Context>) -> HcResult<Option<Entry>> {
    let action_wrapper = ActionWrapper::new(Action::GetEntry(address));
    dispatch_action(&context.action_channel, action_wrapper.clone());
    await!(GetEntryFuture {
        context: context.clone(),
        address: action_wrapper,
    })
}

/// GetEntryFuture resolves to a HcResult<Entry>.
/// Tracks the state of the network module
pub struct GetEntryFuture {
    context: Arc<Context>,
    address: Address,
}

impl Unpin for GetEntryFuture {}

impl Future for GetEntryFuture {
    type Output = HcResult<Entry>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        let state = self.context.state().unwrap().network();
        if state.network.is_none() || state.dna_hash.is_none() || state.agent_id.is_none() {
            return Poll::Ready(Err(HolochainError::IoError(
                "Network not initialized".to_string(),
            )));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        match state.get_entry_results.get(&self.address) {
            Some(Some(result)) => Poll::Ready(result),
            _ => Poll::Pending,
        }
    }
}
