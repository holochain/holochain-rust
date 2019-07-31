use crate::{
    action::RespondGetPayload,
    network::{
        query::{GetLinksNetworkQuery, GetLinksNetworkResult,GetLinksQueryConfiguration},
        actions::get_entry::{get_entry,GetMethod},
    },
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::author_entry::author_entry
};

use holochain_core_types::{
    time::Timeout,
    entry::Entry,
    error::HolochainError,
    link::{link_data::LinkData, LinkActionKind},
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
pub fn invoke_remove_link(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match LinkEntriesArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            context.log(format!(
                "err/zome: invoke_remove_link failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let top_chain_header_option = context.state().unwrap().agent().top_chain_header();

    let top_chain_header = match top_chain_header_option {
        Some(top_chain) => top_chain,
        None => {
            context.log(format!(
                "err/zome: invoke_link_entries failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            ));
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
    let config = GetLinksQueryConfiguration
    {
        headers : false
    };
    let method = GetMethod::Link(get_links_args.clone(),GetLinksNetworkQuery::Links(config));
    let response_result = context.block_on(get_entry(
        context.clone(),
        method,
        Timeout::default()
    ));
    if response_result.is_err()
    {
        context.log("err/zome : Could not get links for remove_link method");
        ribosome_error_code!(WorkflowFailed)
    }
    else
    {
        let response = response_result.expect("Could not get response");
        let links_result = match response
        {
            RespondGetPayload::Links((query,_,_)) => Ok(query),
            RespondGetPayload::Entry(_) => Err(HolochainError::ErrorGeneric("Could not get links for type".to_string()))
        };
        if links_result.is_err() {
            context.log("err/zome : Could not get links for remove_link method");
            ribosome_error_code!(WorkflowFailed)
        } else {
            let links = links_result.expect("This is supposed to not fail");
            let links = match links {
                GetLinksNetworkResult::Links(links) => links,
                _ => return ribosome_error_code!(WorkflowFailed),
            };
            let filtered_links = links
                .into_iter()
                .filter(|link_for_filter|&link_for_filter.target == link.target())
                .map(|s|s.address)
                .collect::<Vec<_>>();

            let entry = Entry::LinkRemove((link_remove, filtered_links));

            // Wait for future to be resolved
            let result: Result<(), HolochainError> = context
                .block_on(author_entry(&entry, None, &context, &vec![]))
                .map(|_| ());

            runtime.store_result(result)
        }
    }
    
}
