extern crate futures;
extern crate serde_json;
use action::{Action, ActionWrapper};
use context::Context;
use futures::{Async, Future, future};
use holochain_core_types::{error::HolochainError};
use instance::dispatch_action;
use std::sync::Arc;


fn get_dna_and_agent(context: &Arc<Context>) -> Result<(String, String), HolochainError> {
    let state = context.state()
        .ok_or("Network::start() could not get application state".to_string())?;
    let agent = state.agent().get_agent(&context)?;
    let agent_id = agent.key;

    let dna = state.nucleus().dna().ok_or("Network::start() called without DNA".to_string())?;
    let dna_hash = base64::encode(&dna.multihash()?);
    Ok((dna_hash, agent_id))
}
/// InitNetwork Action Creator
pub fn initialize_network(context: &Arc<Context>) -> Box<dyn Future<Item = (), Error = HolochainError>>  {
    match get_dna_and_agent(context) {
        Err(error) => return Box::new(future::err(error)),
        Ok((dna_hash, agent_id)) => {
            let action_wrapper = ActionWrapper::new(
                Action::InitNetwork((
                    context.network_config.clone(),
                    dna_hash,
                    agent_id,
                )
                ));
            dispatch_action(&context.action_channel, action_wrapper.clone());

            Box::new(InitNetworkFuture {
                context: context.clone(),
            })
        }
    }
}

pub struct InitNetworkFuture {
    context: Arc<Context>,
}

impl Future for InitNetworkFuture {
    type Item = ();
    type Error = HolochainError;

    fn poll(
        &mut self,
        cx: &mut futures::task::Context<'_>,
    ) -> Result<Async<Self::Item>, Self::Error> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().wake();
        if let Some(state) = self.context.state() {
            if state.network().network.is_some() || state.network().dna_hash.is_some() ||  state.network().agent_id.is_some() {
                Ok(futures::Async::Ready(()))
            } else {
                Ok(futures::Async::Pending)
            }
        } else {
            Ok(futures::Async::Pending)
        }
    }
}
