use std::sync::Arc;
use crate::context::Context;
use crate::nucleus::ZomeFnCall;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use crate::action::GetLinksKey;
use crate::network::direct_message::DirectMessage;
use std::convert::TryInto;
use holochain_core_types::entry::Entry;
use holochain_core_types::error::HolochainError;

fn address_to_content_and_type(address: &Address, context: Arc<Context>) -> Result<(String, String), HolochainError> {
    let raw_content = context.dht_storage.read()?.fetch(address)??;
    let maybe_entry: Result<Entry, _> = raw_content.clone().try_into();
    if let Ok(entry) = maybe_entry {
        let mut entry_type = entry.entry_type().to_string();
        let content = match entry {
            Entry::Dna(_)=> String::from("DNA omitted"),
            Entry::AgentId(agent_id) => agent_id.nick,
            Entry::LinkAdd(link) | Entry::LinkRemove((link, _)) => format!(
                "({}#{})\n\t{} => {}",
                link.link.link_type(),
                link.link.tag(),
                link.link.base(),
                link.link.target(),
            ),
            Entry::App(app_type, app_value) => {
                entry_type = app_type.to_string();
                app_value.to_string()
            }
            _ => entry.content().to_string(),
        };
        Ok((entry_type, content))
    } else {
        Ok((String::from("UNKNOWN"), raw_content.to_string()))
    }
}

pub fn state_dump(context: Arc<Context>) {
    let (nucleus, network, dht) = {
        let state_lock = context.state().expect("No state?!");
        ((*state_lock.nucleus()).clone(), (*state_lock.network()).clone(), (*state_lock.dht()).clone())
    };


    let running_calls: Vec<ZomeFnCall> = nucleus.zome_calls
        .into_iter()
        .filter(|(_, result)| result.is_none())
        .map(|(call, _)| call)
        .collect();

    let get_entry_flows: Vec<Address> = network.get_entry_with_meta_results
        .into_iter()
        .filter(|(_, result)| result.is_none())
        .map(|(key, _)| key.address.clone())
        .collect();

    let get_links_flows: Vec<GetLinksKey> = network.get_links_results
        .into_iter()
        .filter(|(_, result)| result.is_none())
        .map(|(key, _)| key)
        .collect();

    let validation_package_flows: Vec<Address> = network.get_validation_package_results
        .into_iter()
        .filter(|(_, result)| result.is_none())
        .map(|(address, _)| address)
        .collect();

    let direct_message_flows: Vec<(String, DirectMessage)> = network.direct_message_connections
        .into_iter()
        .map(|(s, dm)| (s.clone(), dm.clone()))
        .collect();

    let pending_validation_strings = nucleus
        .pending_validations
        .keys()
        .map(|pending_validation_key| {
            let maybe_content = address_to_content_and_type(&pending_validation_key.address, context.clone());
            maybe_content
                .map(|(content_type, content)| format!("<{}> [{}] {}: {}", pending_validation_key.workflow.to_string(), content_type, pending_validation_key.address.to_string(), content))
                .unwrap_or_else(|err| format!("<{}> [UNKNOWN] {}: Error trying to get type/content: {}", pending_validation_key.workflow.to_string(), pending_validation_key.address.to_string(), err))
        })
        .collect::<Vec<String>>();

    let holding_strings = dht
        .get_all_held_entry_addresses()
        .iter()
        .map(|address| {
            let maybe_content = address_to_content_and_type(address, context.clone());
            maybe_content
                .map(|(content_type, content)|format!("* [{}] {}: {}", content_type, address.to_string(), content))
                .unwrap_or_else(|err| format!("* [UNKNOWN] {}: Error trying to get type/content: {}", address.to_string(), err))
        })
        .collect::<Vec<String>>();

    let debug_dump = format!(r#"
=============STATE DUMP===============

Nucleus:
========
Running zome calls: {calls:?}
-------------------
Pending validations:
{validations}
--------------------

Network:
--------
Running GET ENTRY flows: {entry_flows:?}
------------------------
Running GET LINKS flows: {links_flows:?}
------------------------
Running VALIDATION PACKAGE requests: {validation_packages:?}
------------------------------------
Running DIRECT MESSAGES: {direct_messages:?}

Dht:
====
Holding:
{holding_list}
--------
    "#,
    calls = running_calls,
    validations = pending_validation_strings.join("\n"),
    entry_flows = get_entry_flows,
    links_flows = get_links_flows,
    validation_packages = validation_package_flows,
    direct_messages = direct_message_flows,
    holding_list = holding_strings.join("\n"));

    log_info!(context, "debug/state_dump: {}", debug_dump);
}