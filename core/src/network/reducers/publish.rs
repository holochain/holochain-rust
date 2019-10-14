use crate::{
    action::ActionWrapper,
    network::{
        actions::ActionResponse,
        entry_aspect::EntryAspect,
        entry_with_header::{fetch_entry_with_header, EntryWithHeader},
        reducers::send,
        state::NetworkState,
    },
    state::State,
};
use holochain_core_types::{
    crud_status::CrudStatus,
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
};
use lib3h_protocol::{
    data_types::{EntryData, ProvidedEntryData},
    protocol_client::Lib3hClientProtocol,
};

use holochain_persistence_api::cas::content::{Address, AddressableContent};

/// Send to network a PublishDhtData message
fn publish_entry(
    network_state: &mut NetworkState,
    entry_with_header: &EntryWithHeader,
) -> Result<(), HolochainError> {
    send(
        network_state,
        Lib3hClientProtocol::PublishEntry(ProvidedEntryData {
            space_address: network_state.dna_address.clone().unwrap().into(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: entry_with_header.entry.address().clone(),
                aspect_list: vec![EntryAspect::Content(
                    entry_with_header.entry.clone(),
                    entry_with_header.header.clone(),
                )
                .into()],
            },
        }),
    )
}

/// Send to network a publish request for either delete or update aspect information
fn publish_update_delete_meta(
    network_state: &mut NetworkState,
    orig_entry_address: Address,
    crud_status: CrudStatus,
    entry_with_header: &EntryWithHeader,
) -> Result<(), HolochainError> {
    // publish crud-status

    let aspect = match crud_status {
        CrudStatus::Modified => EntryAspect::Update(
            entry_with_header.entry.clone(),
            entry_with_header.header.clone(),
        ),
        CrudStatus::Deleted => EntryAspect::Deletion(entry_with_header.header.clone()),
        crud => {
            return Err(HolochainError::ErrorGeneric(format!(
                "Unexpeced CRUD variant {:?}",
                crud
            )));
        }
    };

    send(
        network_state,
        Lib3hClientProtocol::PublishEntry(ProvidedEntryData {
            space_address: network_state.dna_address.clone().unwrap().into(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: orig_entry_address,
                aspect_list: vec![aspect.into()],
            },
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
    let (base, aspect) = match entry_with_header.entry.clone() {
        Entry::LinkAdd(link_data) => (
            link_data.link().base().clone(),
            EntryAspect::LinkAdd(link_data, entry_with_header.header.clone()),
        ),
        Entry::LinkRemove((link_data, links_to_remove)) => (
            link_data.link().base().clone(),
            EntryAspect::LinkRemove(
                (link_data, links_to_remove),
                entry_with_header.header.clone(),
            ),
        ),
        _ => {
            return Err(HolochainError::ErrorGeneric(format!(
                "Received bad entry type. Expected Entry::LinkAdd/Remove received {:?}",
                entry_with_header.entry,
            )));
        }
    };
    send(
        network_state,
        Lib3hClientProtocol::PublishEntry(ProvidedEntryData {
            space_address: network_state.dna_address.clone().unwrap().into(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: base,
                aspect_list: vec![aspect.into()],
            },
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
            match entry_with_header.header.link_update_delete() {
                Some(modified_entry) => publish_update_delete_meta(
                    network_state,
                    modified_entry,
                    CrudStatus::Modified,
                    &entry_with_header.clone(),
                ),
                None => Ok(()),
            }
        }),
        EntryType::LinkAdd => publish_entry(network_state, &entry_with_header)
            .and_then(|_| publish_link_meta(network_state, &entry_with_header)),
        EntryType::LinkRemove => publish_entry(network_state, &entry_with_header)
            .and_then(|_| publish_link_meta(network_state, &entry_with_header)),
        EntryType::Deletion => publish_entry(network_state, &entry_with_header).and_then(|_| {
            match entry_with_header.header.link_update_delete() {
                Some(modified_entry) => publish_update_delete_meta(
                    network_state,
                    modified_entry,
                    CrudStatus::Deleted,
                    &entry_with_header.clone(),
                ),
                None => Ok(()),
            }
        }),
        _ => Err(HolochainError::NotImplemented(
            format!("reduce_publish_inner not implemented for {}", entry_with_header.entry.entry_type()),
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
    use holochain_core_types::entry::test_entry;
    use holochain_persistence_api::cas::content::AddressableContent;

    #[test]
    pub fn reduce_publish_test() {
        let context = test_context("alice", None);
        let store = test_store(context.clone());

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::Publish(entry.address()));

        store.reduce(action_wrapper);
    }
}
