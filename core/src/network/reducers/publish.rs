use crate::{
    action::ActionWrapper,
    context::Context,
    network::{actions::ActionResponse, state::NetworkState, util},
};
use boolinator::*;
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    chain_header::ChainHeader,
    entry::{entry_type::EntryType, Entry, ToEntry},
    error::HolochainError,
    link::link_add::LinkAddEntry,
};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{DhtData, DhtMetaData, ProtocolWrapper},
};
use std::sync::Arc;

fn publish_entry(
    network_state: &mut NetworkState,
    entry: &Entry,
    header: &ChainHeader,
) -> Result<(), HolochainError> {
    let entry_with_header = util::EntryWithHeader::from((entry.clone(), header.clone()));

    let data = DhtData {
        msg_id: "?".to_string(),
        dna_hash: network_state.dna_hash.clone().unwrap(),
        agent_id: network_state.agent_id.clone().unwrap(),
        address: entry.address().to_string(),
        content: serde_json::from_str(&serde_json::to_string(&entry_with_header).unwrap()).unwrap(),
    };

    network_state
        .network
        .as_mut()
        .map(|network| {
            network
                .lock()
                .unwrap()
                .send(ProtocolWrapper::PublishDht(data).into())
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .expect("Network has to be Some because of check above")
}

fn publish_link(
    network_state: &mut NetworkState,
    entry: &Entry,
    header: &ChainHeader,
) -> Result<(), HolochainError> {
    let entry_with_header = util::EntryWithHeader::from((entry.clone(), header.clone()));
    let link_add_entry = LinkAddEntry::from_entry(&entry);
    let link = link_add_entry.link().clone();

    //let header = maybe_header.unwrap();
    let data = DhtMetaData {
        msg_id: "?".to_string(),
        dna_hash: network_state.dna_hash.clone().unwrap(),
        agent_id: network_state.agent_id.clone().unwrap(),
        address: link.base().to_string(),
        attribute: String::from("link"),
        content: serde_json::from_str(&serde_json::to_string(&entry_with_header).unwrap()).unwrap(),
    };

    network_state
        .network
        .as_mut()
        .map(|network| {
            network
                .lock()
                .unwrap()
                .send(ProtocolWrapper::PublishDhtMeta(data).into())
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .expect("Network has to be Some because of check above")
}

fn inner(
    context: &Arc<Context>,
    network_state: &mut NetworkState,
    address: &Address,
) -> Result<(), HolochainError> {
    (network_state.network.is_some()
        && network_state.dna_hash.is_some() & network_state.agent_id.is_some())
    .ok_or("Network not initialized".to_string())?;

    let (entry, header) = util::entry_with_header(&address, &context)?;

    match entry.entry_type() {
        EntryType::App(_) => publish_entry(network_state, &entry, &header),
        EntryType::LinkAdd => publish_entry(network_state, &entry, &header)
            .and_then(|_| publish_link(network_state, &entry, &header)),
        _ => Err(HolochainError::NotImplemented),
    }
}

pub fn reduce_publish(
    context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let address = unwrap_to!(action => crate::action::Action::Publish);

    let result = inner(&context, network_state, &address);

    network_state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::Publish(match result {
            Ok(_) => Ok(address.clone()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}

#[cfg(test)]
mod tests {

    use crate::{
        action::{Action, ActionWrapper},
        instance::tests::test_context,
        state::test_store,
    };
    use holochain_core_types::{cas::content::AddressableContent, entry::test_entry};

    #[test]
    pub fn reduce_publish_test() {
        let context = test_context("alice");
        let store = test_store(context.clone());

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::Publish(entry.address()));

        store.reduce(context.clone(), action_wrapper);
    }

}
