use crate::{
    wasm_engine::{api::ZomeApiResult, Runtime},
    network::{
        actions::query::{query, QueryMethod},
        query::{
            GetLinksNetworkQuery, GetLinksNetworkResult, GetLinksQueryConfiguration,
            NetworkQueryResult,
        },
    },
    workflows::author_entry::author_entry,
    NEW_RELIC_LICENSE_KEY,
};

use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    link::{link_data::LinkData, LinkActionKind},
    time::Timeout,
    network::query::GetLinkFromRemoteData,
};
use holochain_wasm_utils::api_serialization::{
    get_links::{GetLinksArgs, GetLinksOptions},
    link_entries::LinkEntriesArgs,
};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::GetLinks function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: GetLinksArgs
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_remove_link(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match LinkEntriesArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            log_error!(
                context,
                "zome: invoke_remove_link failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let top_chain_header_option = context.state().unwrap().agent().top_chain_header();

    let top_chain_header = match top_chain_header_option {
        Some(top_chain) => top_chain,
        None => {
            log_error!(
                context,
                "zome: invoke_link_entries failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
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
    let config = GetLinksQueryConfiguration { headers: false };
    let method = QueryMethod::Link(get_links_args, GetLinksNetworkQuery::Links(config));
    let response_result = context.block_on(query(context.clone(), method, Timeout::default()));
    if response_result.is_err() {
        log_error!("zome : Could not get links for remove_link method.");
        ribosome_error_code!(WorkflowFailed)
    } else {
        let response = response_result.expect("Could not get response");
        let links_result = match response {
            NetworkQueryResult::Links(query, _, _) => Ok(query),
            NetworkQueryResult::Entry(_) => Err(HolochainError::ErrorGeneric(
                "Could not get links for type".to_string(),
            )),
        };

        if let Ok(GetLinksNetworkResult::Links(links)) = links_result {
            let filtered_links = links
                .into_iter()
                .map(|GetLinkFromRemoteData {link_add_address, ..}| link_add_address)
                .collect::<Vec<_>>();

            let entry = Entry::LinkRemove((link_remove, filtered_links));

            // Wait for future to be resolved
            let result: Result<(), HolochainError> = context
                .block_on(author_entry(&entry, None, &context, &vec![]))
                .map(|_| ());

            runtime.store_result(result)
        } else {
            log_error!(context, "zome : Could not get links for remove_link method");
            ribosome_error_code!(WorkflowFailed)
        }
    }
}
