use crate::{
    network::{
        actions::get_links::get_links,
        query::{GetLinksNetworkQuery, GetLinksNetworkResult},
    },
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::{author_entry::author_entry, get_entry_result::get_entry_result_workflow},
};

use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    link::{link_data::LinkData, LinkActionKind},
};
use holochain_wasm_utils::api_serialization::{
    get_entry::{GetEntryArgs, GetEntryOptions, GetEntryResultType},
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
            context.log_error(format!(
                "zome: invoke_remove_link failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let top_chain_header_option = context.state().unwrap().agent().top_chain_header();

    let top_chain_header = match top_chain_header_option {
        Some(top_chain) => top_chain,
        None => {
            context.log_error(format!(
                "zome: invoke_link_entries failed to deserialize LinkEntriesArgs: {:?}",
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
    let links_result = context.block_on(get_links(
        context.clone(),
        &get_links_args,
        GetLinksNetworkQuery::Links,
    ));
    if links_result.is_err() {
        context.log_error("zome : Could not get links for remove_link method");
        ribosome_error_code!(WorkflowFailed)
    } else {
        let links = links_result.expect("This is supposed to not fail");
        let links = match links {
            GetLinksNetworkResult::Links(links) => links,
            _ => return ribosome_error_code!(WorkflowFailed),
        };
        let filtered_links = links
            .into_iter()
            .map(|link_crud| link_crud.0)
            .filter(|link_address| {
                context
                    .block_on(get_entry_result_workflow(
                        &context,
                        &GetEntryArgs {
                            address: link_address.clone().clone(),
                            options: GetEntryOptions::default(),
                        },
                    ))
                    .map(|get_entry_result| match get_entry_result.result {
                        GetEntryResultType::Single(single_item) => single_item
                            .entry
                            .map(|entry| match entry {
                                Entry::LinkAdd(link_data) => {
                                    link_data.link().target() == link.target() 
                                }
                                _ => false,
                            })
                            .unwrap_or(false),
                        _ => false,
                    })
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        let entry = Entry::LinkRemove((link_remove, filtered_links));

        // Wait for future to be resolved
        let result: Result<(), HolochainError> = context
            .block_on(author_entry(&entry, None, &context, &vec![]))
            .map(|_| ());

        runtime.store_result(result)
    }
}
