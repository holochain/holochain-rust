use crate::{
    context::Context, network::actions::get_links::get_links,
    workflows::get_entry_result::get_entry_result_workflow,
};

use holochain_core_types::{
    chain_header::ChainHeader, entry::Entry, error::HolochainError, link::link_data::LinkData,
    crud_status::CrudStatus,
};
use holochain_wasm_utils::api_serialization::{
    get_entry::{GetEntryArgs, GetEntryOptions, GetEntryResultType::Single},
    get_links::{GetLinksArgs, GetLinksResult, LinksResult, LinksStatusRequestKind},
};
use std::sync::Arc;

pub async fn get_link_result_workflow<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResult, HolochainError> {
    let links = await!(get_link_add_entries(context, link_args))?;
    let link_results = links
        .into_iter()
        .filter(|link_entry_crud|{
            match link_args.options.status_request
            {
                LinksStatusRequestKind::All => true,
                LinksStatusRequestKind::Live => link_entry_crud.2 == CrudStatus::Live,
                _ => link_entry_crud.2 == CrudStatus::Deleted
            }
        })
        .map(|link_entry_crud|{
            LinksResult
            {
                address : link_entry_crud.0.link().target().clone(),
                headers : link_entry_crud.1.clone(),
                status : link_entry_crud.2.clone(),
                tag : link_entry_crud.0.link().tag().clone()
            }
        })
        .collect::<Vec<LinksResult>>();

    Ok(GetLinksResult::new(link_results))
}

pub async fn get_link_add_entries<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<Vec<(LinkData, Vec<ChainHeader>, CrudStatus)>, HolochainError> {
    let links_caches = await!(get_links(
        context.clone(),
        link_args.entry_address.clone(),
        link_args.link_type.clone(),
        link_args.tag.clone(),
        link_args.options.timeout.clone()
    ))?;

    let (links_result, get_links_error): (Vec<_>, Vec<_>) = links_caches
        .iter()
        .map(|s| {
            let get_entry_args = GetEntryArgs {
                address: s.0.clone(),
                options: GetEntryOptions {
                    entry: true,
                    headers: link_args.options.headers,
                    timeout: link_args.options.timeout.clone(),
                    ..GetEntryOptions::default()
                },
            };
            let entry_result =
                context.block_on(get_entry_result_workflow(&context.clone(), &get_entry_args));
            entry_result
                .map(|link_entry_result| match link_entry_result.result {
                    Single(entry_type) => entry_type
                        .entry
                        .clone()
                        .map(|unwrapped_type| match unwrapped_type {
                            Entry::LinkAdd(link_data) => Ok((link_data, entry_type.headers,s.1.clone())),
                            _ => Err(HolochainError::ErrorGeneric(
                                "Wrong entry type retrieved".to_string(),
                            )),
                        })
                        .unwrap_or(Err(HolochainError::ErrorGeneric(
                            "Could not obtain Entry".to_string(),
                        ))),
                    _ => Err(HolochainError::ErrorGeneric(
                        "Status Kind Of Lastest Requested".to_string(),
                    )),
                })
                .unwrap_or(Err(HolochainError::ErrorGeneric(
                    "expected entry of type link".to_string(),
                )))
        })
        .partition(Result::is_ok);

    if get_links_error.is_empty() {
        Ok(links_result
            .into_iter()
            .map(|s| s.unwrap())
            .collect::<Vec<_>>())
    } else {
        Err(HolochainError::ErrorGeneric(format!(
            "Could not get links: {:?}",
            get_links_error
        )))
    }
}


