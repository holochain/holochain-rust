use crate::{
    action::ActionWrapper,

    context::Context,
    network::{actions::ActionResponse, state::NetworkState, util},
};
use boolinator::*;
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::{entry_type::EntryType},
    error::HolochainError,
};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{DhtMetaData, ProtocolWrapper},
};
use std::sync::Arc;

fn inner(
    context: Arc<Context>,
    state: &mut NetworkState,
    address: &Address
) -> Result<(), HolochainError> {
    (state.network.is_none() || state.dna_hash.is_none() || state.agent_id.is_none())
        .ok_or("Network not initialized".to_string())?;

    let (entry, header) = util::entry_with_header(&address, &context)?;

    (entry.entry_type() != &EntryType::LinkAdd)
        .ok_or("Given address not a LinkAdd entry".to_string())?;

    let entry_with_header = util::EntryWithHeader::from((entry.clone(), header));

    //let header = maybe_header.unwrap();
    let data = DhtMetaData {
        msg_id: "?".to_string(),
        dna_hash: state.dna_hash.clone().unwrap(),
        agent_id: state.agent_id.clone().unwrap(),
        address: entry.address().to_string(),
        attribute: String::from("link"),
        content: serde_json::from_str(&serde_json::to_string(&entry_with_header).unwrap()).unwrap(),
    };

    state.network
        .as_mut()
        .map(|network| {
            network.lock()
                .unwrap()
                .send(ProtocolWrapper::PublishDhtMeta(data).into())
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .expect("Network has to be Some because of check above")

}
pub fn reduce_publish_link(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let address = unwrap_to!(action => crate::action::Action::PublishLink);

    let result = inner(context, state, &address);

    state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::PublishLink(match result {
            Ok(_) => Ok(address.clone()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}
