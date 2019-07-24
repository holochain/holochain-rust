use std::sync::Arc;
use crate::context::Context;
use crate::nucleus::ZomeFnCall;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use crate::action::GetLinksKey;
use crate::network::direct_message::DirectMessage;
use std::convert::TryInto;
use holochain_core_types::entry::Entry;

pub fn state_dump(context: Arc<Context>) {
    let state_lock = context.state().expect("No state?!");

    let running_calls: Vec<ZomeFnCall> = state_lock.nucleus().zome_calls
        .iter()
        .filter(|(_, result)| result.is_none())
        .map(|(call, _)| call)
        .cloned()
        .collect();

    let get_entry_flows: Vec<Address> = state_lock.network().get_entry_with_meta_results
        .iter()
        .filter(|(_, result)| result.is_none())
        .map(|(key, _)| key.address.clone())
        .collect();

    let get_links_flows: Vec<GetLinksKey> = state_lock.network().get_links_results
        .iter()
        .filter(|(_, result)| result.is_none())
        .map(|(key, _)| key)
        .cloned()
        .collect();

    let validation_package_flows: Vec<Address> = state_lock.network().get_validation_package_results
        .iter()
        .filter(|(_, result)| result.is_none())
        .map(|(address, _)| address)
        .cloned()
        .collect();

    let direct_message_flows: Vec<(String, DirectMessage)> = state_lock.network().direct_message_connections
        .iter()
        .map(|(s, dm)| (s.clone(), dm.clone()))
        .collect();

    let holding_strings = state_lock
        .dht()
        .get_all_held_entry_addresses()
        .iter()
        .map(|address| {
            let raw_content = context.dht_storage.read().unwrap().fetch(address).unwrap().unwrap();
            let maybe_entry: Result<Entry, _> = raw_content.clone().try_into();
            let (content_type, content) = if let Ok(entry) = maybe_entry {
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
                (entry_type, content)
            } else {
                (String::from("UNKNOWN"), raw_content.to_string())
            };
            format!("* {}: [{}] {}", address.to_string(), content_type, content)
        })
        .collect::<Vec<String>>();

    let debug_dump = format!(r#"
=============STATE DUMP===============

Nucleus:
========
Running zome calls: {calls:?}
-------------------
Pending validations: {validations:?}
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
    validations = state_lock.nucleus().pending_validations.keys(),
    entry_flows = get_entry_flows,
    links_flows = get_links_flows,
    validation_packages = validation_package_flows,
    direct_messages = direct_message_flows,
    holding_list = holding_strings.join("\n"));

    context.log(format!("debug/state_dump: {}", debug_dump));
}