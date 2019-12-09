use crate::{
    action::ActionWrapper,
    network::{
        actions::NetworkActionResponse, chain_pair::ChainPair, entry_aspect::EntryAspect,
        reducers::send, state::NetworkState,
    },
    state::State,
};
use chrono::{offset::FixedOffset, DateTime};
use holochain_core_types::{
    crud_status::CrudStatus,
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
};
use holochain_json_api::json::JsonString;
use lib3h_protocol::{
    data_types::{EntryAspectData, EntryData, ProvidedEntryData},
    protocol_client::Lib3hClientProtocol,
};

use crate::network::actions::Response;
use holochain_persistence_api::cas::content::{Address, AddressableContent};

pub fn entry_data_to_entry_aspect_data(ea: &EntryAspect) -> EntryAspectData {
    let type_hint = ea.type_hint();
    let aspect_address = ea.address();
    let ts: DateTime<FixedOffset> = ea.header().timestamp().into();
    let aspect_json: JsonString = ea.into();
    EntryAspectData {
        type_hint,
        aspect_address: aspect_address.into(),
        aspect: aspect_json.to_bytes().into(),
        publish_ts: ts.timestamp() as u64,
    }
}

/// Send to network a PublishDhtData message
fn publish_entry(
    network_state: &mut NetworkState,
    chain_pair: &ChainPair,
) -> Result<(), HolochainError> {
    send(
        network_state,
        Lib3hClientProtocol::PublishEntry(ProvidedEntryData {
            space_address: network_state.dna_address.clone().unwrap().into(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: chain_pair.entry().address().into(),
                aspect_list: vec![entry_data_to_entry_aspect_data(&EntryAspect::Content(
                    chain_pair.entry().clone(),
                    chain_pair.header().clone(),
                ))],
            },
        }),
    )
}

/// Send to network a publish request for either delete or update aspect information
fn publish_update_delete_meta(
    network_state: &mut NetworkState,
    orig_entry_address: Address,
    crud_status: CrudStatus,
    chain_pair: &ChainPair,
) -> Result<(), HolochainError> {
    // publish crud-status

    let aspect = match crud_status {
        CrudStatus::Modified => {
            EntryAspect::Update(chain_pair.entry().clone(), chain_pair.header().clone())
        }
        CrudStatus::Deleted => EntryAspect::Deletion(chain_pair.header().clone()),
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
                entry_address: orig_entry_address.into(),
                aspect_list: vec![entry_data_to_entry_aspect_data(&aspect)],
            },
        }),
    )?;

    // publish crud-link if there is one
    Ok(())
}

/// Send to network a PublishMeta message holding a link metadata to `chain_pair`
fn publish_link_meta(
    network_state: &mut NetworkState,
    chain_pair: &ChainPair,
) -> Result<(), HolochainError> {
    let (base, aspect) = match chain_pair.entry().clone() {
        Entry::LinkAdd(link_data) => (
            link_data.link().base().clone(),
            EntryAspect::LinkAdd(link_data, chain_pair.header().clone()),
        ),
        Entry::LinkRemove((link_data, links_to_remove)) => (
            link_data.link().base().clone(),
            EntryAspect::LinkRemove((link_data, links_to_remove), chain_pair.header().clone()),
        ),
        _ => {
            return Err(HolochainError::ErrorGeneric(format!(
                "Received bad entry type. Expected Entry::LinkAdd/Remove received {:?}",
                chain_pair.entry(),
            )));
        }
    };
    send(
        network_state,
        Lib3hClientProtocol::PublishEntry(ProvidedEntryData {
            space_address: network_state.dna_address.clone().unwrap().into(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: base.into(),
                aspect_list: vec![entry_data_to_entry_aspect_data(&aspect)],
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

    let chain_pair = ChainPair::fetch_chain_pair(&address, root_state)?;

    match chain_pair.entry().entry_type() {
        EntryType::AgentId => publish_entry(network_state, &chain_pair),
        EntryType::App(_) => {
            publish_entry(network_state, &chain_pair).and_then(|_| {
                match chain_pair.header().link_update_delete() {
                    Some(modified_entry) => publish_update_delete_meta(
                        network_state,
                        modified_entry,
                        CrudStatus::Modified,
                        &chain_pair.clone(),
                    ),
                    None => Ok(()),
                }
            })
        }
        EntryType::LinkAdd => publish_entry(network_state, &chain_pair)
            .and_then(|_| publish_link_meta(network_state, &chain_pair)),
        EntryType::LinkRemove => publish_entry(network_state, &chain_pair)
            .and_then(|_| publish_link_meta(network_state, &chain_pair)),
        EntryType::Deletion => publish_entry(network_state, &chain_pair).and_then(|_| {
            match chain_pair.header().link_update_delete() {
                Some(modified_entry) => publish_update_delete_meta(
                    network_state,
                    modified_entry,
                    CrudStatus::Deleted,
                    &chain_pair.clone(),
                ),
                None => Ok(()),
            }
        }),
        _ => Err(HolochainError::NotImplemented(format!(
            "reduce_publish_inner not implemented for {}",
            chain_pair.entry().entry_type()
        ))),
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
        Response::from(NetworkActionResponse::Publish(match result {
            Ok(_) => Ok(address.clone()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        })),
    );
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        action::{Action, ActionWrapper},
        instance::tests::test_context,
        state::test_store,
    };
    use chrono::{offset::FixedOffset, DateTime};
    use holochain_core_types::{chain_header::test_chain_header, entry::test_entry};
    use holochain_persistence_api::cas::content::AddressableContent;
    use lib3h_protocol::types::AspectHash;

    #[test]
    pub fn reduce_publish_test() {
        let context = test_context("alice", None);
        let store = test_store(context.clone());

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::Publish(entry.address()));

        store.reduce(action_wrapper);
    }

    #[test]
    fn can_convert_into_entry_aspect_data() {
        let chain_header = test_chain_header();
        let aspect = EntryAspect::Header(chain_header.clone());
        let aspect_data: EntryAspectData = entry_data_to_entry_aspect_data(&aspect);
        let aspect_json: JsonString = aspect.clone().into();
        let ts: DateTime<FixedOffset> = chain_header.timestamp().into();
        assert_eq!(aspect_data.type_hint, aspect.type_hint());
        assert_eq!(
            aspect_data.aspect_address,
            AspectHash::from(aspect.address())
        );
        assert_eq!(*aspect_data.aspect, aspect_json.to_bytes());
        assert_eq!(aspect_data.publish_ts, ts.timestamp() as u64);
    }
}
