use crate::{
    action::ActionWrapper,
    context::Context,
    network::{
        actions::ActionResponse,
        reducers::{initialized, send},
        state::NetworkState,
        util,
    },
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    chain_header::ChainHeader,
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
};
use holochain_net_connection::{
    protocol_wrapper::{DhtData, DhtMetaData, ProtocolWrapper},
};
use std::sync::Arc;

fn publish_entry(
    network_state: &mut NetworkState,
    entry: &Entry,
    header: &ChainHeader,
) -> Result<(), HolochainError> {
    let entry_with_header = util::EntryWithHeader::from((entry.clone(), header.clone()));

    send(network_state, ProtocolWrapper::PublishDht(DhtData {
        msg_id: "?".to_string(),
        dna_hash: network_state.dna_hash.clone().unwrap(),
        agent_id: network_state.agent_id.clone().unwrap(),
        address: entry.address().to_string(),
        content: serde_json::from_str(&serde_json::to_string(&entry_with_header).unwrap()).unwrap(),
    }))
}

fn publish_link(
    network_state: &mut NetworkState,
    entry: &Entry,
    header: &ChainHeader,
) -> Result<(), HolochainError> {
    let entry_with_header = util::EntryWithHeader::from((entry.clone(), header.clone()));
    let link_add = match entry {
        Entry::LinkAdd(link_add) => link_add,
        _ => {
            return Err(HolochainError::ErrorGeneric(format!(
                "Received bad entry type. Expected Entry::LinkAdd received {:?}",
                entry,
            )));
        }
    };
    let link = link_add.link().clone();

    send(network_state, ProtocolWrapper::PublishDhtMeta(DhtMetaData {
        msg_id: "?".to_string(),
        dna_hash: network_state.dna_hash.clone().unwrap(),
        agent_id: network_state.agent_id.clone().unwrap(),
        address: link.base().to_string(),
        attribute: String::from("link"),
        content: serde_json::from_str(&serde_json::to_string(&entry_with_header).unwrap()).unwrap(),
    }))
}

fn inner(
    context: &Arc<Context>,
    network_state: &mut NetworkState,
    address: &Address,
) -> Result<(), HolochainError> {
    initialized(network_state)?;

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
