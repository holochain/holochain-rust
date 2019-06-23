use crate::{
    action::{Action, ActionWrapper, NetworkSettings},
    context::{get_dna_and_agent, Context},
    instance::dispatch_action,
    network::{actions::publish::publish, handler::create_handler},
};
use futures::{
    task::{LocalWaker, Poll},
    Future,
};
use holochain_core_types::error::HcResult;
#[cfg(test)]
use holochain_persistence_api::cas::content::Address;
use std::{pin::Pin, sync::Arc};

/// Creates a network proxy object and stores DNA and agent hash in the network state.
pub async fn initialize_network(context: &Arc<Context>) -> HcResult<()> {
    let (dna_address, agent_id) = await!(get_dna_and_agent(context))?;
    let handler = create_handler(&context, dna_address.to_string());
    let network_settings = NetworkSettings {
        p2p_config: context.p2p_config.clone(),
        dna_address,
        agent_id: agent_id.clone(),
        handler,
    };
    let action_wrapper = ActionWrapper::new(Action::InitNetwork(network_settings));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    await!(InitNetworkFuture {
        context: context.clone(),
    })?;

    await!(publish(agent_id.clone().into(), context))?;

    Ok(())
}

#[cfg(test)]
pub async fn initialize_network_with_spoofed_dna(
    dna_address: Address,
    context: &Arc<Context>,
) -> HcResult<()> {
    let (_, agent_id) = await!(get_dna_and_agent(context))?;
    let handler = create_handler(&context, dna_address.to_string());
    let network_settings = NetworkSettings {
        p2p_config: context.p2p_config.clone(),
        dna_address,
        agent_id,
        handler,
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
