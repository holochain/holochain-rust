use crate::context::Context;
use std::sync::Arc;
use crate::state_dump::{StateDump, address_to_content_and_type};

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
