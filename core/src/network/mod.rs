pub mod actions;
pub mod handler;
pub mod state;

use crate::{
    action::{Action, ActionWrapper, NetworkReduceFn},
    agent::chain_header,
    context::Context,
    network::{
        state::NetworkState,
        handler::create_handler,
    },
};
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent},
        //storage::ContentAddressableStorage
    },
    entry::{Entry, SerializedEntry},
    error::HolochainError,
};
use holochain_net::p2p_network::{P2pNetwork};
use holochain_net_connection::{
    //NetResult,
    net_connection::NetConnection,
    protocol_wrapper::{
        DhtData,
        ProtocolWrapper, TrackAppData,
    }
};
use std::{
    convert::TryInto,
    sync::{Arc, Mutex}
};

fn reduce_init(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
){
    let action = action_wrapper.action();
    let (_, dna_hash, agent_id) = unwrap_to!(action => Action::InitNetwork);
    let mut network = P2pNetwork::new(
        create_handler(&context),
        &context.network_config
    ).unwrap();

    let _ = network.send(
        ProtocolWrapper::TrackApp(
                TrackAppData{
                    dna_hash: dna_hash.clone(),
                    agent_id: agent_id.clone(),
                })
            .into()
    )
        .and_then(|_| {
            state.network = Some(Arc::new(Mutex::new(network)));
            state.dna_hash = Some(dna_hash.clone());
            state.agent_id = Some(agent_id.clone());
            Ok(())
        });
}

fn entry_from_cas(address: &Address, context: &Arc<Context>,) -> Result<Entry, HolochainError>{
    let json = context.file_storage.read().unwrap().fetch(address)?
        .ok_or("Entry not found".to_string())?;
    let s: SerializedEntry = json.try_into()?;
    Ok(s.into())
}

fn reduce_publish(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {

    if state.network.is_none() || state.dna_hash.is_none() ||  state.agent_id.is_none() {
        return;
    }

    let action = action_wrapper.action();
    let address = unwrap_to!(action => Action::Publish);

    let result = entry_from_cas(address, &context);
    if result.is_err() {
        return;
    };

    let (entry, maybe_header) = result.map(|entry|{
            let header = chain_header(&entry, &context);
            (entry, header)
        })
        .unwrap();

    if maybe_header.is_none() {
        // We don't have the entry in our source chain?!
        // Don't publish
        return;
    }

    //let header = maybe_header.unwrap();
    let data = DhtData {
        msg_id: "?".to_string(),
        dna_hash: state.dna_hash.clone().unwrap(),
        agent_id: state.agent_id.clone().unwrap(),
        address: entry.address().to_string(),
        content: serde_json::from_str(&entry.content().to_string()).unwrap(),
    };

    let _ = match state.network {
        None => unreachable!(),
        Some(ref network) => {
            network.lock()
                .unwrap()
                .send(ProtocolWrapper::PublishDht(data).into())
        }
    };

}

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<NetworkReduceFn> {
    match action_wrapper.action() {
        Action::Publish(_) => Some(reduce_publish),
        Action::InitNetwork(_) => Some(reduce_init),
        _ => None,
    }
}


pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<NetworkState>,
    action_wrapper: &ActionWrapper,
) -> Arc<NetworkState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: NetworkState = (*old_state).clone();
            f(context, &mut new_state, &action_wrapper);
            Arc::new(new_state)
        }
        None => old_state,
    }
}