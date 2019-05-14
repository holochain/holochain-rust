use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::author_entry::author_entry,
    workflows::{get_link_result::get_link_result_workflow,get_entry_result::get_entry_result_workflow},
};

use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    link::{link_data::LinkData, LinkActionKind},
};
use holochain_wasm_utils::api_serialization::{link_entries::LinkEntriesArgs,get_links::{GetLinksArgs,GetLinksOptions},get_entry::{GetEntryArgs,GetEntryResultType,GetEntryOptions}};
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

    let link = input.to_link();
    let link_remove = LinkData::from_link(&link, LinkActionKind::REMOVE);
    let input = GetLinksArgs{
        entry_address : link.base().clone(),
        tag : link.tag().clone(),
        options : GetLinksOptions::default()
    };
    let links_result = context.block_on(get_link_result_workflow(&context, &input));
    if links_result.is_err()
    {
        context.log("err/zome : Could not get links for remove_link method");
        return ribosome_error_code!(ArgumentDeserializationFailed);
    }

    let links = links_result.expect("This is supposed to not fail").addresses();
    let filtered_links = links.into_iter().filter(|link_address|{
        context.block_on(get_entry_result_workflow(&context,&GetEntryArgs{
            address : link_address.clone().clone(),
            options : GetEntryOptions::default()
        })).map(|get_entry_result|{
            match get_entry_result.result
            {
                GetEntryResultType::Single(single_item) =>
                {
                   single_item.entry.map(|entry|{
                        match entry
                        {
                            Entry::LinkAdd(link_data) => {
                                link_data.link().target() == link.target()
                            },
                            _ => false
                        }
                    }).unwrap_or(false)
                },
                _ => false
            }
        }).unwrap_or(false)
    }).collect::<Vec<_>>();

    let entry = Entry::LinkRemove((link_remove,filtered_links));

    // Wait for future to be resolved
    let result: Result<(), HolochainError> = context
        .block_on(author_entry(&entry, None, &context, &vec![]))
        .map(|_| ());

    runtime.store_result(result)
}


