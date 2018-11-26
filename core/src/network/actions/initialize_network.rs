extern crate futures;
extern crate serde_json;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{Future, task::{Poll, LocalWaker}};
use holochain_core_types::{error::HolochainError};
use std::{
    pin::{Pin, Unpin},
    sync::Arc,
};


async fn get_dna_and_agent(context: &Arc<Context>) -> Result<(String, String), HolochainError> {
    let state = context.state()
        .ok_or("Network::start() could not get application state".to_string())?;
    let agent_state = state.agent();

    let agent = await!(agent_state.get_agent(&context))?;
    let agent_id = agent.key;

    let dna = state.nucleus().dna().ok_or("Network::start() called without DNA".to_string())?;
    let dna_hash = base64::encode(&dna.multihash()?);
    Ok((dna_hash, agent_id))
}
/// InitNetwork Action Creator
pub async fn initialize_network(context: &Arc<Context>) -> Result<(),HolochainError>  {
    let (dna_hash, agent_id) = await!(get_dna_and_agent(context))?;
    let action_wrapper = ActionWrapper::new(
        Action::InitNetwork((
            context.network_config.clone(),
            dna_hash,
            agent_id,
        )
        ));
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
    type Output = Result<(),HolochainError>;

    fn poll(
        self: Pin<&mut Self>,
        lw: &LocalWaker,
    ) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        if let Some(state) = self.context.state() {
            if state.network().network.is_some() || state.network().dna_hash.is_some() ||  state.network().agent_id.is_some() {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
