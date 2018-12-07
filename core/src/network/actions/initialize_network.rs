extern crate futures;
extern crate serde_json;
use crate::{
    action::{Action, ActionWrapper},
    context::{Context,get_dna_and_agent},
    instance::dispatch_action,
};
use futures::{
    task::{LocalWaker, Poll},
    Future,
};
use holochain_core_types::error::HcResult;
use std::{
    pin::{Pin, Unpin},
    sync::Arc,
};


/// InitNetwork Action Creator
pub async fn initialize_network(context: &Arc<Context>) -> HcResult<()> {
    let (dna_hash, agent_id) = await!(get_dna_and_agent(context))?;
    let action_wrapper = ActionWrapper::new(Action::InitNetwork((
        context.network_config.clone(),
        dna_hash,
        agent_id,
    )));
    dispatch_action(&context.action_channel, action_wrapper.clone());

    await!(InitNetworkFuture {
        context: context.clone(),
    })
}

pub struct InitNetworkFuture {
    context: Arc<Context>,
}

impl Unpin for InitNetworkFuture {}

impl Future for InitNetworkFuture {
    type Output = HcResult<()>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        if let Some(state) = self.context.state() {
            if state.network().network.is_some()
                || state.network().dna_hash.is_some()
                || state.network().agent_id.is_some()
            {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
