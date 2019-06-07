use crate::{
    action::ActionWrapper,
    network::{
        actions::ActionResponse,
        entry_with_header::{fetch_entry_with_header, EntryWithHeader},
        reducers::send,
        state::NetworkState,
    },
    state::State,
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    crud_status::CrudStatus,
    eav::Attribute,
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
};
use holochain_net::connection::json_protocol::{DhtMetaData, EntryData, JsonProtocol};

/// Send to network a PublishDhtData message
fn publish_entry(
    network_state: &mut NetworkState,
    entry_with_header: &EntryWithHeader,
) -> Result<(), HolochainError> {
    send(
        network_state,
        JsonProtocol::PublishEntry(EntryData {
            dna_address: network_state.dna_address.clone().unwrap(),
            provider_agent_id: network_state.agent_id.clone().unwrap(),
            entry_address: entry_with_header.entry.address().clone(),
            entry_content: serde_json::from_str(
                &serde_json::to_string(&entry_with_header).unwrap(),
            )
            .unwrap(),
        }),
    )
}

/// Send to network:
///  - a PublishDhtMeta message for the crud-status
///  - a PublishDhtMeta message for the crud-link
fn publish_update_delete_meta(
    network_state: &mut NetworkState,
    entry_address: Address,
    crud_status: String,
    entry_with_header: &EntryWithHeader,
) -> Result<(), HolochainError> {
    // publish crud-status
    send(
        network_state,
        JsonProtocol::PublishMeta(DhtMetaData {
            dna_address: network_state.dna_address.clone().unwrap(),
            provider_agent_id: network_state.agent_id.clone().unwrap(),
            entry_address: entry_address.clone(),
            attribute: crud_status,
            content_list: vec![serde_json::from_str(
                &serde_json::to_string(&entry_with_header).unwrap(),
            )
            .unwrap()],
        }),
    )?;

    // publish crud-link if there is one
    Ok(())
}

/// Send to network a PublishMeta message holding a link metadata to `entry_with_header`
fn publish_link_meta(
    network_state: &mut NetworkState,
    entry_with_header: &EntryWithHeader,
) -> Result<(), HolochainError> {
    let (link_type, link_attribute) = match entry_with_header.entry.clone() {
        Entry::LinkAdd(link_add_entry) => (link_add_entry, Attribute::Link),
        Entry::LinkRemove(link_remove) => (link_remove, Attribute::LinkRemove),
        _ => {
            return Err(HolochainError::ErrorGeneric(format!(
                "Received bad entry type. Expected Entry::LinkAdd received {:?}",
                entry_with_header.entry,
            )));
        }
    };
    let link = link_type.link().clone();

    send(
        network_state,
        JsonProtocol::PublishMeta(DhtMetaData {
            dna_address: network_state.dna_address.clone().unwrap(),
            provider_agent_id: network_state.agent_id.clone().unwrap(),
            entry_address: link.base().clone(),
            attribute: link_attribute.to_string(),
            content_list: vec![serde_json::from_str(
                &serde_json::to_string(&entry_with_header).unwrap(),
            )
            .unwrap()],
        }),
    )
}

fn reduce_publish_inner(
    network_state: &mut NetworkState,
    root_state: &State,
    address: &Address,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    let entry_with_header = fetch_entry_with_header(&address, root_state)?;
    match entry_with_header.entry.entry_type() {
        EntryType::AgentId => publish_entry(network_state, &entry_with_header),
        EntryType::App(_) => publish_entry(network_state, &entry_with_header).and_then(|_| {
            if entry_with_header.header.link_update_delete().is_some() {
                publish_update_delete_meta(
                    network_state,
                    entry_with_header.entry.address(),
                    String::from(CrudStatus::Modified),
                    &entry_with_header.clone(),
                )
            } else {
                Ok(())
            }
        }),
        EntryType::LinkAdd => publish_entry(network_state, &entry_with_header)
            .and_then(|_| publish_link_meta(network_state, &entry_with_header)),
        EntryType::LinkRemove => publish_entry(network_state, &entry_with_header)
            .and_then(|_| publish_link_meta(network_state, &entry_with_header)),
        EntryType::Deletion => publish_entry(network_state, &entry_with_header).and_then(|_| {
            publish_update_delete_meta(
                network_state,
                entry_with_header.entry.address(),
                String::from(CrudStatus::Deleted),
                &entry_with_header.clone(),
            )
        }),
        _ => Err(HolochainError::NotImplemented(
            "reduce_publish_inner".into(),
        )),
    }
}

pub fn reduce_publish(
    network_state: &mut NetworkState,
    root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let address = unwrap_to!(action => crate::action::Action::Publish);

    let result = reduce_publish_inner(network_state, root_state, &address);
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
        let context = test_context("alice", None);
        let store = test_store(context.clone());

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::Publish(entry.address()));

        store.reduce(action_wrapper);
    }

}
