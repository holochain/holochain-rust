use crate::{
    context::Context, network::actions::get_links::get_links,
    workflows::get_entry_result::get_entry_result_workflow,
};

use holochain_core_types::{
    chain_header::ChainHeader, entry::Entry, error::HolochainError, link::link_data::LinkData,
    crud_status::CrudStatus,
    time::Timeout,
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
                                    status : link.2
        })
        .collect::<Vec<LinksResult>>();

    Ok(GetLinksResult::new(link_results))
}

pub async fn get_link_add_entries<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<Vec<(LinkData, Vec<ChainHeader>)>, HolochainError> {
) -> Result<Vec<(LinkData, Option<EntryWithMetaAndHeader>, CrudStatus)>, HolochainError> {
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
                address: s.clone(),
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
                            Entry::LinkAdd(link_data) => Ok((link_data, entry_type.headers)),
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

pub fn get_latest_entry(
    context: Arc<Context>,
    address: Address,
    timeout: Timeout,
) -> Result<Option<EntryWithMetaAndHeader>, HolochainError> {
    let entry_with_meta_and_header =
        context.block_on(get_entry_with_meta_workflow(&context, &address, &timeout))?;
    entry_with_meta_and_header
        .map(|entry_meta_header| {
            if let Some(maybe_link_update) =
                entry_meta_header.entry_with_meta.maybe_link_update_delete
            {
                get_latest_entry(context.clone(), maybe_link_update, timeout)
            } else {
                entry_meta_header
                    .headers
                    .first()
                    .map(|first_chain_header| {
                        first_chain_header
                            .link_update_delete()
                            .map(|link| {
                                context.block_on(get_entry_with_meta_workflow(
                                    &context, &link, &timeout,
                                ))
                            })
                            .unwrap_or(Ok(Some(entry_meta_header.clone())))
                    })
                    .unwrap_or(Err(HolochainError::ErrorGeneric(
                        "disjointed link update".to_string(),
                    )))
            }
        })
        .unwrap_or(Ok(None))
}
