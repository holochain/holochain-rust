use crate::context::Context;
use holochain_core_types::{entry::Entry, error::HolochainError};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use std::{convert::TryInto, sync::Arc};
use crate::state_dump::StateDump;

fn address_to_content_and_type(
    address: &Address,
    context: Arc<Context>,
) -> Result<(String, String), HolochainError> {
    let raw_content = context.dht_storage.read()?.fetch(address)??;
    let maybe_entry: Result<Entry, _> = raw_content.clone().try_into();
    if let Ok(entry) = maybe_entry {
        let mut entry_type = entry.entry_type().to_string();
        let content = match entry {
            Entry::Dna(_) => String::from("DNA omitted"),
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
    let dump = StateDump::from(context.clone());

    let pending_validation_strings = dump.pending_validations
        .iter()
        .map(|pending_validation| {
            let maybe_content =
                address_to_content_and_type(&pending_validation.address, context.clone());
            maybe_content
                .map(|(content_type, content)| {
                    format!(
                        "<{}> [{}] {}: {}",
                        pending_validation.workflow.to_string(),
                        content_type,
                        pending_validation.address.to_string(),
                        content
                    )
                })
                .unwrap_or_else(|err| {
                    format!(
                        "<{}> [UNKNOWN] {}: Error trying to get type/content: {}",
                        pending_validation.workflow.to_string(),
                        pending_validation.address.to_string(),
                        err
                    )
                })
        })
        .collect::<Vec<String>>();

    let holding_strings = dump.held_entries
        .iter()
        .map(|address| {
            let maybe_content = address_to_content_and_type(address, context.clone());
            maybe_content
                .map(|(content_type, content)| {
                    format!("* [{}] {}: {}", content_type, address.to_string(), content)
                })
                .unwrap_or_else(|err| {
                    format!(
                        "* [UNKNOWN] {}: Error trying to get type/content: {}",
                        address.to_string(),
                        err
                    )
                })
        })
        .collect::<Vec<String>>();

    let debug_dump = format!(
        r#"
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
Running query flows: {flows:?}
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
        calls = dump.running_calls,
        validations = pending_validation_strings.join("\n"),
        flows = dump.query_flows,
        validation_packages = dump.validation_package_flows,
        direct_messages = dump.direct_message_flows,
        holding_list = holding_strings.join("\n")
    );

    log_info!(context, "debug/state_dump: {}", debug_dump);
}
