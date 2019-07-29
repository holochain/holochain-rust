use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    entry::CanPublish,
    instance::dispatch_action,
    network::handler::{get_content_aspect, get_meta_aspects},
};
use holochain_core_types::error::HcResult;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use lib3h_protocol::data_types::{EntryListData, GetListData};
use snowflake::ProcessUniqueId;
use std::{collections::HashMap, sync::Arc, thread};

pub fn handle_get_authoring_list(get_list_data: GetListData, context: Arc<Context>) {
    thread::Builder::new()
        .name(format!(
            "handle_authoring_list/{}",
            ProcessUniqueId::new().to_string()
        ))
        .spawn(move || {
            let mut address_map = HashMap::new();
            for entry in get_all_public_chain_entries(context.clone()) {
                address_map.insert(
                    entry.clone(),
                    get_all_aspect_addresses(&entry, context.clone())
                        .expect("Error getting entry aspects of authoring list"),
                );
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
    let chain = context.state().unwrap().agent().chain_store();
    let top_header = context.state().unwrap().agent().top_chain_header();
    chain
        .iter(&top_header)
        .filter(|ref chain_header| chain_header.entry_type().can_publish(&context))
        .map(|chain_header| chain_header.entry_address().clone())
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
