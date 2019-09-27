use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    entry::CanPublish,
    instance::dispatch_action,
    network::handler::{get_content_aspect, get_meta_aspects},
};
use holochain_core_types::{
    error::HcResult,
    entry::Entry,
};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use lib3h_protocol::data_types::{EntryListData, GetListData};
use snowflake::ProcessUniqueId;
use std::{collections::HashMap, sync::Arc, thread};
use crate::network::entry_aspect::EntryAspect;
use crate::agent::state::create_new_chain_header;

pub fn handle_get_authoring_list(get_list_data: GetListData, context: Arc<Context>) {
    thread::Builder::new()
        .name(format!(
            "handle_authoring_list/{}",
            ProcessUniqueId::new().to_string()
        ))
        .spawn(move || {
            let mut address_map = HashMap::new();
            for entry in get_all_public_chain_entries(context.clone()) {
                let content_aspect = get_content_aspect(&entry, context.clone())
                    .expect("Must be able to get content aspect of entry that is in our source chain");
                address_map.insert(
                    entry.clone(),
                    vec![content_aspect.address()]
                );
            }

            // chain header entries also should be communicated on the authoring list
            // In future make this depend if header publishing is enabled
            let state = context.state()
                .expect("There must be a state in context when we are responding to a HandleGetAuthoringEntryList");
            for chain_header_entry in get_all_chain_header_entries(context.clone()) {
                let address = chain_header_entry.address();
                let header_entry_header = create_new_chain_header(
                    &chain_header_entry,
                    &state.agent(),
                    &*state,
                    &None,
                    &Vec::new(),
                ).expect("Must be able to create dummy header header when responding to HandleGetAuthoringEntryList");
                let content_aspect = EntryAspect::Content(
                    chain_header_entry,
                    header_entry_header,
                );
                address_map.insert(address, vec![content_aspect.address()]);
            }

            let action = Action::RespondAuthoringList(EntryListData {
                space_address: get_list_data.space_address,
                provider_agent_id: get_list_data.provider_agent_id,
                request_id: get_list_data.request_id,
                address_map,
            });
            dispatch_action(context.action_channel(), ActionWrapper::new(action));
        })
        .expect("Could not spawn thread for creating of authoring list");
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
    chain
        .map(|chain_header| Entry::ChainHeader(chain_header))
        .collect()
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
    thread::Builder::new()
        .name(format!(
            "handle_gossip_list/{}",
            ProcessUniqueId::new().to_string()
        ))
        .spawn(move || {
            let mut address_map = HashMap::new();
            let holding_list = {
                let state = context
                    .state()
                    .expect("No state present when trying to respond with gossip list");
                state.dht().get_all_held_entry_addresses().clone()
            };

            for entry in holding_list {
                address_map.insert(
                    entry.clone(),
                    get_all_aspect_addresses(&entry, context.clone())
                        .expect("Error getting entry aspects of authoring list"),
                );
            }

            let action = Action::RespondGossipList(EntryListData {
                space_address: get_list_data.space_address,
                provider_agent_id: get_list_data.provider_agent_id,
                request_id: get_list_data.request_id,
                address_map,
            });
            dispatch_action(context.action_channel(), ActionWrapper::new(action));
        })
        .expect("Could not spawn thread for creating of gossip list");
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::workflows::author_entry::author_entry;
    use crate::nucleus::actions::tests::*;
    use holochain_core_types::{
        entry::{Entry, test_entry_with_value},
    };
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

        assert_eq!(
            get_all_chain_header_entries(context),
            header_entries,
        )

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

        assert!(get_all_chain_header_entries(context.clone()).iter().all(|chain_header| {
            get_all_aspect_addresses(&chain_header.address(), context.clone()).is_ok()
        }));
    }
}
