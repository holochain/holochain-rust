use crate::{
    action::{Action, ActionWrapper},
    instance::dispatch_action,
};
use crossbeam_channel::Sender;
use futures::{future::Future, task::Poll};
use holochain_core_types::error::{HcResult, HolochainError};
use holochain_locksmith::RwLock;

use crate::state::State;
use std::{pin::Pin, sync::Arc};

/// Shutdown the network
/// This tells the network to untrack this instance and then stops the network thread
/// and sets the P2pNetwork instance in the state to None.
pub async fn shutdown(
    state: Arc<RwLock<State>>,
    action_channel: Sender<ActionWrapper>,
) -> HcResult<()> {
    if state.read().unwrap().network().initialized().is_ok() {
        let action_wrapper = ActionWrapper::new(Action::ShutdownNetwork);
        dispatch_action(&action_channel, action_wrapper.clone());
        ShutdownFuture { state }.await
    } else {
        Err(HolochainError::ErrorGeneric(
            "Tried to shutdown network that was never initialized".to_string(),
        ))
    }
}

pub struct ShutdownFuture {
    state: Arc<RwLock<State>>,
}

impl Future for ShutdownFuture {
    type Output = HcResult<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        self.state
            .try_read()
            .map(|state| {
                if state.network().network.is_some() {
                    //
                    // TODO: connect the waker to state updates for performance reasons
                    // See: https://github.com/holochain/holochain-rust/issues/314
                    //
                    cx.waker().clone().wake();
                    Poll::Pending
                } else {
                    Poll::Ready(Ok(()))
                }
            })
            .unwrap_or(Poll::Pending)
    }
}
