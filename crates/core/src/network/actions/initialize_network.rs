use crate::{
    action::{Action, ActionWrapper, NetworkSettings},
    context::{get_dna_and_agent, Context},
    instance::dispatch_action,
    network::handler::create_handler,
};
use futures::{task::Poll, Future};
use holochain_core_types::error::HcResult;
use holochain_persistence_api::cas::content::Address;
use std::{pin::Pin, sync::Arc};
use holochain_net::p2p_config::P2pConfig;
use holochain_net::connection::net_connection::NetHandler;

/// Creates a network proxy object and stores DNA and agent hash in the network state.
pub async fn initialize_network(
    p2p_config: P2pConfig,
    dna_address: Address,
    agent_id: String,
    handler: NetHandler,
    context: &Arc<Context>
) -> HcResult<()> {
    let network_settings = NetworkSettings {
        p2p_config,
        dna_address,
        agent_id,
        handler,
    };
    let action_wrapper = ActionWrapper::new(Action::InitNetwork(network_settings));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    log_debug!(context, "waiting for network");
    InitNetworkFuture {
        context: context.clone(),
    }
    .await?;

    Ok(())
}

#[cfg(test)]
pub async fn initialize_network_with_spoofed_dna(
    p2p_config: P2pConfig,
    dna_address: Address,
    agent_id: String,
    handler: NetHandler,
    context: &Arc<Context>
) -> HcResult<()> {
    let (_, agent_id) = get_dna_and_agent(context).await?;
    let network_settings = NetworkSettings {
        p2p_config,
        dna_address,
        agent_id,
        handler,
    };
    let action_wrapper = ActionWrapper::new(Action::InitNetwork(network_settings));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    InitNetworkFuture {
        context: context.clone(),
    }
    .await
}

pub struct InitNetworkFuture {
    context: Arc<Context>,
}

impl Future for InitNetworkFuture {
    type Output = HcResult<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("InitializeNetworkFuture") {
            return Poll::Ready(Err(err));
        }
        //

        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        if let Some(state) = self.context.try_state() {
            if state.network().network.is_some()
                && state.network().dna_address.is_some()
                && state.network().agent_id.is_some()
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
