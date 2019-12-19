use crate::{
    action::{Action, ActionWrapper},
    agent::state::create_entry_with_header_for_header,
    context::Context,
    dht::aspect_map::{AspectMap, AspectMapBare},
    entry::CanPublish,
    instance::dispatch_action,
    network::{entry_aspect::EntryAspect, handler::get_content_aspect},
};
use holochain_core_types::entry::Entry;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use im::HashSet;
use lib3h_protocol::{
    data_types::{EntryListData, GetListData},
    types::{AspectHash, EntryHash},
};
use std::sync::Arc;

pub fn handle_get_authoring_list(get_list_data: GetListData, context: Arc<Context>) {
    let c = context.clone();
    let closure = async move || {
        let address_map = create_authoring_map(context.clone());

        let action = Action::RespondAuthoringList(EntryListData {
            space_address: get_list_data.space_address,
            provider_agent_id: get_list_data.provider_agent_id,
            request_id: get_list_data.request_id,
            address_map: address_map.into(),
        });
        dispatch_action(context.action_channel(), ActionWrapper::new(action));
    };

    let future = closure();
    c.spawn_task(future);
}

fn create_authoring_map(context: Arc<Context>) -> AspectMap {
    let mut address_map: AspectMapBare = AspectMapBare::new();
    for entry_address in get_all_public_chain_entries(context.clone()) {
        // 1. For every public chain entry we definitely add the content aspect:
        let content_aspect = get_content_aspect(&entry_address, context.clone())
            .expect("Must be able to get content aspect of entry that is in our source chain");

        address_map
            .entry(EntryHash::from(entry_address.clone()))
            .or_insert_with(HashSet::new)
            .insert(AspectHash::from(content_aspect.address()));

        // 2. Then we might need to add a meta aspect as well depending on what kind of
        //    entry this is.

        // So we first unwrap the entry it self from the content aspect:
        let (entry, header) = match content_aspect {
            EntryAspect::Content(entry, header) => (entry, header),
            _ => panic!("get_content_aspect must return only EntryAspect::Content"),
        };

        // And then we deduce the according base entry and meta aspect from that entry
        // and its header:
        let maybe_meta_aspect = match entry {
            Entry::App(app_type, app_value) => header.link_update_delete().map(|updated_entry| {
                (
                    updated_entry,
                    EntryAspect::Update(Entry::App(app_type, app_value), header),
                )
            }),
            Entry::LinkAdd(link_data) => Some((
                link_data.link.base().clone(),
                EntryAspect::LinkAdd(link_data, header),
            )),
            Entry::LinkRemove((link_data, addresses)) => Some((
                link_data.link.base().clone(),
                EntryAspect::LinkRemove((link_data, addresses), header),
            )),
            Entry::Deletion(_) => Some((
                header.link_update_delete().expect(""),
                EntryAspect::Deletion(header),
            )),
            _ => None,
        };

        if let Some((base_address, meta_aspect)) = maybe_meta_aspect {
            address_map
                .entry(EntryHash::from(base_address.clone()))
                .or_insert_with(HashSet::new)
                .insert(AspectHash::from(meta_aspect.address()));
        }
    }

    // chain header entries also should be communicated on the authoring list
    // In future make this depend on if header publishing is enabled
    let state = context.state().expect(
        "There must be a state in context when we are responding to a HandleGetAuthoringEntryList",
    );
    // So we iterate over all our source chain headers
    for chain_header in context.state().unwrap().agent().iter_chain() {
        // Create an entry that represents the header
        match create_entry_with_header_for_header(&state, chain_header.clone()) {
            Err(e) => {
                log_error!(
                    context,
                    "Could not create virtual header for header. Error: {:?}",
                    e
                );
                continue;
            }
            Ok(chain_entry_with_header) => {
                let entry_hash = chain_entry_with_header.entry.address();
                let content_aspect = EntryAspect::Content(
                    chain_entry_with_header.entry,
                    chain_entry_with_header.header,
                );
                let aspect_hash = AspectHash::from(content_aspect.address());
                address_map
                    .entry(entry_hash.into())
                    .or_insert_with(|| {
                        let mut set = HashSet::new();
                        set.insert(aspect_hash.clone());
                        set
                    })
                    .insert(aspect_hash);
            }
        }
    }
    address_map.into()
}

fn get_all_public_chain_entries(context: Arc<Context>) -> Vec<Address> {
    let chain = context.state().unwrap().agent().iter_chain();
    chain
        .filter(|ref chain_header| chain_header.entry_type().can_publish(&context))
        .map(|chain_header| chain_header.entry_address().clone())
        .collect()
}

pub fn handle_get_gossip_list(get_list_data: GetListData, context: Arc<Context>) {
    let c = context.clone();
    let closure = async move || {
        let state = context
            .state()
            .expect("No state present when trying to respond with gossip list");
        let address_map = state.dht().get_holding_map().clone();

        let action = Action::RespondGossipList(EntryListData {
            space_address: get_list_data.space_address,
            provider_agent_id: get_list_data.provider_agent_id,
            request_id: get_list_data.request_id,
            address_map: address_map.into(),
        });
        dispatch_action(context.action_channel(), ActionWrapper::new(action));
    };
    let future = closure();
    c.spawn_task(future);
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::nucleus::actions::tests::*;

    #[test]
    fn test_can_get_authoring_list() {
        let mut dna = test_dna();
        dna.uuid = "test_can_get_authoring_list".to_string();
        let (_instance, context) = instance_by_name("jill", dna, None);
        let authoring_map = create_authoring_map(context);
        assert_eq!(authoring_map.bare().len(), 3);
    }

    #[test]
    fn test_can_get_holding_list() {
        let mut dna = test_dna();
        dna.uuid = "test_can_get_holding_list".to_string();
        let (_instance, context) = instance_by_name("jill", dna, None);
        let authoring_map = create_authoring_map(context);
        // to start with holding = authoring
        assert_eq!(authoring_map.bare().len(), 3);
    }
}
