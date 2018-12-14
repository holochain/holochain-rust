use crate::{
    action::ActionWrapper,
    context::Context,
    network::{
        actions::ActionResponse,
        entry_with_header::{fetch_entry_with_header, EntryWithHeader},
        reducers::send,
        state::NetworkState,
    },
    nucleus::actions::get_entry::get_entry_crud_meta_from_dht,
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    crud_status::{CrudStatus, LINK_NAME, STATUS_NAME},
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
};
use holochain_net_connection::protocol_wrapper::{DhtData, DhtMetaData, ProtocolWrapper};
use std::sync::Arc;

fn publish_entry(
    network_state: &mut NetworkState,
    entry_with_header: &EntryWithHeader,
) -> Result<(), HolochainError> {
    //let entry_with_header = util::EntryWithHeader::from((entry.clone(), header.clone()));

    send(
        network_state,
        ProtocolWrapper::PublishDht(DhtData {
            msg_id: "?".to_string(),
            dna_hash: network_state.dna_hash.clone().unwrap(),
            agent_id: network_state.agent_id.clone().unwrap(),
            address: entry_with_header.entry.address().to_string(),
            content: serde_json::from_str(&serde_json::to_string(&entry_with_header).unwrap())
                .unwrap(),
        }),
    )
}

fn publish_crud_meta(
    network_state: &mut NetworkState,
    entry_address: Address,
    crud_status: CrudStatus,
    crud_link: Option<Address>,
) -> Result<(), HolochainError> {
    // publish crud-status
    send(
        network_state,
        ProtocolWrapper::PublishDhtMeta(DhtMetaData {
            msg_id: "?".to_string(),
            dna_hash: network_state.dna_hash.clone().unwrap(),
            agent_id: network_state.agent_id.clone().unwrap(),
            address: entry_address.to_string(),
            attribute: STATUS_NAME.to_string(),
            content: serde_json::from_str(&serde_json::to_string(&crud_status).unwrap()).unwrap(),
        }),
    )?;

    // publish crud-link if there is one
    if crud_link.is_none() {
        return Ok(());
    }
    send(
        network_state,
        ProtocolWrapper::PublishDhtMeta(DhtMetaData {
            msg_id: "?".to_string(),
            dna_hash: network_state.dna_hash.clone().unwrap(),
            agent_id: network_state.agent_id.clone().unwrap(),
            address: entry_address.to_string(),
            attribute: LINK_NAME.to_string(),
            content: serde_json::from_str(&serde_json::to_string(&crud_link.unwrap()).unwrap())
                .unwrap(),
        }),
    )?;
    Ok(())
}

fn publish_link_meta(
    network_state: &mut NetworkState,
    entry_with_header: &EntryWithHeader,
) -> Result<(), HolochainError> {
    let link_add_entry = match entry_with_header.entry.clone() {
        Entry::LinkAdd(link_add_entry) => link_add_entry,
        _ => {
            return Err(HolochainError::ErrorGeneric(format!(
                "Received bad entry type. Expected Entry::LinkAdd received {:?}",
                entry_with_header.entry,
            )));
        }
    };
    let link = link_add_entry.link().clone();

    send(
        network_state,
        ProtocolWrapper::PublishDhtMeta(DhtMetaData {
            msg_id: "?".to_string(),
            dna_hash: network_state.dna_hash.clone().unwrap(),
            agent_id: network_state.agent_id.clone().unwrap(),
            address: link.base().to_string(),
            attribute: String::from("link"),
            content: serde_json::from_str(&serde_json::to_string(&entry_with_header).unwrap())
                .unwrap(),
        }),
    )
}

fn reduce_publish_inner(
    context: &Arc<Context>,
    network_state: &mut NetworkState,
    address: &Address,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    let entry_with_header = fetch_entry_with_header(&address, &context)?;
    let (crud_status, maybe_crud_link) = get_entry_crud_meta_from_dht(context, address.clone())?
        .expect("Entry should have crud-status metadata in DHT.");
    match entry_with_header.entry.entry_type() {
        EntryType::AgentId => publish_entry(network_state, &entry_with_header).and_then(|_| {
            publish_crud_meta(
                network_state,
                entry_with_header.entry.address(),
                crud_status,
                maybe_crud_link,
            )
        }),
        EntryType::App(_) => publish_entry(network_state, &entry_with_header).and_then(|_| {
            publish_crud_meta(
                network_state,
                entry_with_header.entry.address(),
                crud_status,
                maybe_crud_link,
            )
        }),
        EntryType::LinkAdd => publish_entry(network_state, &entry_with_header)
            .and_then(|_| publish_link_meta(network_state, &entry_with_header)),
        EntryType::Deletion => publish_entry(network_state, &entry_with_header).and_then(|_| {
            publish_crud_meta(
                network_state,
                entry_with_header.entry.address(),
                crud_status,
                maybe_crud_link,
            )
        }),
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

    let result = reduce_publish_inner(&context, network_state, &address);
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
