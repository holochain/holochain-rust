use crate::{
    context::Context, network::actions::get_links::get_links,
    workflows::get_entry_result::get_entry_with_meta_workflow,
};

use holochain_core_types::{
    cas::content::Address,
    crud_status::CrudStatus,
    entry::{Entry, EntryWithMeta, EntryWithMetaAndHeader},
    error::HolochainError,
    link::link_data::LinkData,
    time::Timeout,
};
use holochain_wasm_utils::api_serialization::get_links::{
    GetLinksArgs, GetLinksResult, LinksResult, LinksStatusRequestKind,
};
use std::sync::Arc;

pub async fn get_link_result_workflow<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResult, HolochainError> {
    let links = await!(get_link_caches(context, link_args))?;
    let (link_results, errors): (Vec<_>, Vec<_>) = links
        .into_iter()
        .filter(|link_entry_crud|{
            match link_args.options.status_request
            {
                LinksStatusRequestKind::All => true,
                LinksStatusRequestKind::Live => link_entry_crud.2 == CrudStatus::Live,
                _ => link_entry_crud.2 == CrudStatus::Deleted
            }
        })
        .map(|link| {
                        match link.1 {
                            Some(EntryWithMetaAndHeader {
                                entry_with_meta: EntryWithMeta{entry: Entry::LinkAdd(link_data), ..},
                                headers,
                            }) => {
                                let headers = match link_args.options.headers {
                                    true => headers,
                                    false => Vec::new(),
                                };
                                Ok(LinksResult {
                                    address: link_data.link().target().clone(),
                                    headers,
                                    tag: link_data.link().tag().to_string(),
                                    status : link.2
                                })
                            },
                            None => {
                                Err(HolochainError::ErrorGeneric(
                                    format!("Could not get link entry for address stored in the EAV entry {:?}", link),
                                ))
                            }
                            _ => {
                                Err(HolochainError::ErrorGeneric(
                                    format!("Unknown Error retrieveing link. Most likely EAV entry points to non-link entry type"),
                                ))
                            }
                        }
                    },
        )
        .partition(Result::is_ok);

    if errors.is_empty() {
        Ok(GetLinksResult::new(
            link_results.into_iter().map(|s| s.unwrap()).collect(),
        ))
    } else {
        Err(HolochainError::ErrorGeneric(
            "Could not get links".to_string(),
        ))
    }
}

async fn get_link_caches<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
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
            let entry_with_header = context.block_on(get_entry_with_meta_workflow(
                &context.clone(),
                &s.0.clone(),
                &link_args.options.timeout.clone(),
            ));
            entry_with_header
                .map(|link_entry_result| {
                    link_entry_result.clone().map(|link_entry| {
                        match link_entry.entry_with_meta.entry {
                            Entry::LinkAdd(link) => Ok((link, link_entry_result, s.1.clone())),
                            _ => Err(HolochainError::ErrorGeneric(
                                "expected entry of type link".to_string(),
                            )),
                        }
                    })
                })
                .unwrap_or(None)
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
