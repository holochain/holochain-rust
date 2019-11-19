use crate::{
    action::{Action, ActionWrapper},
    agent::state::create_new_chain_header,
    context::Context,
    entry::CanPublish,
    instance::dispatch_action,
    network::{
        entry_aspect::EntryAspect,
        handler::{get_content_aspect, get_meta_aspects},
    },
};
use holochain_core_types::{entry::Entry, error::HcResult};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use lib3h_protocol::{
    data_types::{EntryListData, GetListData},
    types::{AspectHash, EntryHash},
};
use std::{collections::HashMap, sync::Arc};

pub fn handle_get_authoring_list(get_list_data: GetListData, context: Arc<Context>) {
    context.clone().spawn_task(move || {
            let mut address_map: HashMap<EntryHash, Vec<AspectHash>> = HashMap::new();
            for entry_address in get_all_public_chain_entries(context.clone()) {
                let content_aspect = get_content_aspect(&entry_address, context.clone())
                    .expect("Must be able to get content aspect of entry that is in our source chain");
                address_map.insert(
                    EntryHash::from(entry_address.clone()),
                    vec![AspectHash::from(content_aspect.address())]
                );
            }

            // chain header entries also should be communicated on the authoring list
            // In future make this depend if header publishing is enabled
            let state = context.state()
                .expect("There must be a state in context when we are responding to a HandleGetAuthoringEntryList");
            for chain_header_entry in get_all_chain_header_entries(context.clone()) {
                let entry_hash: EntryHash = chain_header_entry.address().into();
                let header_entry_header = create_new_chain_header(
                    &chain_header_entry,
                    &state.agent(),
                    &state,
                    &None,
                    &Vec::new(),
                ).expect("Must be able to create dummy header header when responding to HandleGetAuthoringEntryList");
                let content_aspect = EntryAspect::Content(
                    chain_header_entry,
                    header_entry_header,
                );
                address_map.insert(entry_hash, vec![AspectHash::from(content_aspect.address())]);
            }

            let action = Action::RespondAuthoringList(EntryListData {
                space_address: get_list_data.space_address,
                provider_agent_id: get_list_data.provider_agent_id,
                request_id: get_list_data.request_id,
                address_map,
            });
            dispatch_action(context.action_channel(), ActionWrapper::new(action));
        });
}

fn get_all_public_chain_entries(context: Arc<Context>) -> Vec<Address> {
    let chain = context.state().unwrap().agent().iter_chain();
    chain
        .filter(|ref chain_header| chain_header.entry_type().can_publish(&context))
        .map(|chain_header| chain_header.entry_address().clone())
        .collect()
}

fn get_all_chain_header_entries(context: Arc<Context>) -> Vec<Entry> {
    let chain = context.state().unwrap().agent().iter_chain();
    chain.map(Entry::ChainHeader).collect()
}

fn get_all_aspect_addresses(entry: &Address, context: Arc<Context>) -> HcResult<Vec<Address>> {
    let mut address_list: Vec<Address> = get_meta_aspects(entry, context.clone())?
        .iter()
        .map(|aspect| aspect.address())
        .collect();
    address_list.push(get_content_aspect(entry, context.clone())?.address());
    Ok(address_list)
}

pub fn handle_get_gossip_list(get_list_data: GetListData, context: Arc<Context>) {
    context.clone().spawn_task(move || {
        let mut address_map: HashMap<EntryHash, Vec<AspectHash>> = HashMap::new();
        let holding_list = {
            let state = context
                .state()
                .expect("No state present when trying to respond with gossip list");
            state.dht().get_all_held_entry_addresses().clone()
        };

        for entry_address in holding_list {
            address_map.insert(
                EntryHash::from(entry_address.clone()),
                get_all_aspect_addresses(&entry_address, context.clone())
                    .expect("Error getting entry aspects of authoring list")
                    .iter()
                    .map(|a| AspectHash::from(a))
                    .collect(),
            );
        }

        let action = Action::RespondGossipList(EntryListData {
            space_address: get_list_data.space_address,
            provider_agent_id: get_list_data.provider_agent_id,
            request_id: get_list_data.request_id,
            address_map,
        });
        dispatch_action(context.action_channel(), ActionWrapper::new(action));
    });
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{nucleus::actions::tests::*, workflows::author_entry::author_entry};
    use holochain_core_types::entry::{test_entry_with_value, Entry};
    use holochain_persistence_api::cas::content::AddressableContent;
    use std::{thread, time};

    #[test]
    fn test_can_get_chain_header_list() {
        let mut dna = test_dna();
        dna.uuid = "test_can_get_chain_header_list".to_string();
        let (_instance, context) = instance_by_name("jill", dna, None);

        context
            .block_on(author_entry(
                &test_entry_with_value("{\"stuff\":\"test entry value\"}"),
                None,
                &context,
                &vec![],
            ))
            .unwrap()
            .address();

        thread::sleep(time::Duration::from_millis(500));

        let chain = context.state().unwrap().agent().iter_chain();
        let header_entries: Vec<Entry> = chain.map(|header| Entry::ChainHeader(header)).collect();

        assert_eq!(get_all_chain_header_entries(context), header_entries,)
    }

    #[test]
    fn test_can_get_all_aspect_addr_for_headers() {
        let mut dna = test_dna();
        dna.uuid = "test_can_get_chain_header_list".to_string();
        let (_instance, context) = instance_by_name("jill", dna, None);

        context
            .block_on(author_entry(
                &test_entry_with_value("{\"stuff\":\"test entry value\"}"),
                None,
                &context,
                &vec![],
            ))
            .unwrap()
            .address();

        thread::sleep(time::Duration::from_millis(500));

        assert!(get_all_chain_header_entries(context.clone())
            .iter()
            .all(|chain_header| {
                get_all_aspect_addresses(&chain_header.address(), context.clone()).is_ok()
            }));
    }
}
