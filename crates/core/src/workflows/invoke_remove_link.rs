use crate::{
    network::{
        actions::query::{query, QueryMethod},
        query::{
            GetLinksNetworkQuery, GetLinksNetworkResult, GetLinksQueryConfiguration,
            NetworkQueryResult,
        },
    },
    workflows::author_entry::author_entry,
    
};
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    link::{link_data::LinkData, LinkActionKind},
    time::Timeout,
};
use holochain_wasm_types::{
    get_links::{GetLinksArgs, GetLinksOptions},
    link_entries::LinkEntriesArgs,
};
use holochain_wasmer_host::*;
use crate::wasm_engine::runtime::Runtime;

/// ZomeApiFunction::GetLinks function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: GetLinksArgs
/// Returns an HcApiReturnCode as I64
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_remove_link(
    runtime: &mut Runtime,
    input: LinkEntriesArgs,
) -> Result<(), HolochainError> {
    let context = runtime.context().map_err(|e| WasmError::Zome(e.to_string()))?;
    let top_chain_header_option = context.state().unwrap().agent().top_chain_header();

    let top_chain_header = match top_chain_header_option {
        Some(top_chain) => top_chain,
        None => {
            log_error!(
                context,
                "zome: invoke_link_entries failed to deserialize LinkEntriesArgs: {:?}",
                input
            );
            return Err(WasmError::ArgumentDeserializationFailed)?;
        }
    };

    let link = input.to_link();
    let link_remove = LinkData::from_link(
        &link,
        LinkActionKind::REMOVE,
        top_chain_header,
        context.agent_id.clone(),
    );
    let get_links_args = GetLinksArgs {
        entry_address: link.base().clone(),
        link_type: link.link_type().clone(),
        tag: link.tag().clone(),
        options: GetLinksOptions::default(),
    };
    let config = GetLinksQueryConfiguration::default();
    let method = QueryMethod::Link(get_links_args, GetLinksNetworkQuery::Links(config));
    let response_result = context.block_on(query(context.clone(), method, Timeout::default()));
    if response_result.is_err() {
        log_error!("zome : Could not get links for remove_link method.");
        Err(WasmError::WorkflowFailed)?
    } else {
        let response = response_result.expect("Could not get response");
        let links_result = match response {
            NetworkQueryResult::Links(query, _, _) => Ok(query),
            NetworkQueryResult::Entry(_) => Err(HolochainError::ErrorGeneric(
                "Could not get links for type".to_string(),
            )),
        };
        if links_result.is_err() {
            log_error!(context, "zome : Could not get links for remove_link method");
            Err(WasmError::WorkflowFailed)?
        } else {
            let links = links_result.expect("This is supposed to not fail");
            let links = match links {
                GetLinksNetworkResult::Links(links) => links,
                _ => Err(WasmError::WorkflowFailed)?,
            };
            let filtered_links = links
                .into_iter()
                .filter(|link_for_filter| &link_for_filter.target == link.target())
                .map(|s| s.address)
                .collect::<Vec<_>>();

            let entry = Entry::LinkRemove((link_remove, filtered_links));

            // Wait for future to be resolved
            context
                .block_on(author_entry(&entry, None, &context, &vec![]))
                .map(|_| ())
        }
    }
}
