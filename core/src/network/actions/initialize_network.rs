extern crate futures;
extern crate serde_json;
use crate::{
    action::{Action, ActionWrapper, NetworkSettings},
    context::{get_dna_and_agent, Context},
    instance::dispatch_action,
};
use futures::{
    task::{LocalWaker, Poll},
    Future,
};
use holochain_core_types::error::HcResult;
use std::{pin::Pin, sync::Arc};
#[cfg(test)]
use holochain_core_types::cas::content::Address;

/// Creates a network proxy object and stores DNA and agent hash in the network state.
pub async fn initialize_network(context: &Arc<Context>) -> HcResult<()> {
    let (dna_address, agent_id) = await!(get_dna_and_agent(context))?;
    let network_settings = NetworkSettings {
        config: context.network_config.clone(),
        dna_address,
        agent_id,
    };
    let action_wrapper = ActionWrapper::new(Action::InitNetwork(network_settings));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    await!(InitNetworkFuture {
        context: context.clone(),
    })
}

#[cfg(test)]
pub async fn initialize_network_with_spoofed_dna(
    dna_address: Address,
    context: &Arc<Context>,
) -> HcResult<()> {
    let (_, agent_id) = await!(get_dna_and_agent(context))?;
    let network_settings = NetworkSettings {
        config: context.network_config.clone(),
        dna_address,
        agent_id,
    };
    let action_wrapper = ActionWrapper::new(Action::InitNetwork(network_settings));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    await!(InitNetworkFuture {
        context: context.clone(),
    })
}

pub struct InitNetworkFuture {
    context: Arc<Context>,
}

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
                || state.network().dna_address.is_some()
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
