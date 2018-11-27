use crate::{
    action::ActionWrapper,
    context::Context,
    network::{actions::ActionResponse, state::NetworkState, util},
};
use holochain_core_types::{
    cas::content::AddressableContent,
    error::HolochainError,
};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{DhtData, ProtocolWrapper},
};
use std::sync::Arc;

pub fn reduce_publish(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    if state.network.is_none() || state.dna_hash.is_none() || state.agent_id.is_none() {
        return;
    }

    let action = action_wrapper.action();
    let address = unwrap_to!(action => crate::action::Action::Publish);

    let (entry, header) = match util::entry_with_header(&address, &context) {
        Err(_) => return,
        Ok(x) => x,
    };

    let entry_with_header = util::EntryWithHeader::from((entry.clone(), header));

    //let header = maybe_header.unwrap();
    let data = DhtData {
        msg_id: "?".to_string(),
        dna_hash: state.dna_hash.clone().unwrap(),
        agent_id: state.agent_id.clone().unwrap(),
        address: entry.address().to_string(),
        content: serde_json::from_str(&serde_json::to_string(&entry_with_header).unwrap()).unwrap(),
    };

    let response = match state.network {
        None => unreachable!(),
        Some(ref network) => network
            .lock()
            .unwrap()
            .send(ProtocolWrapper::PublishDht(data).into()),
    };

    state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::Publish(match response {
            Ok(_) => Ok(entry.address().to_owned()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}
