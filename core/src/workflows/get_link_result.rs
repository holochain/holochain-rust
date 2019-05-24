use crate::{
    context::Context, network::actions::get_links::get_links,
    workflows::get_entry_result::get_entry_with_meta_workflow,
};

use holochain_core_types::{
use holochain_core_types::{
    entry::{Entry, EntryWithMeta, EntryWithMetaAndHeader},
    error::HolochainError,
};
use holochain_wasm_utils::api_serialization::get_links::{
    GetLinksArgs, GetLinksResult, LinksResult, LinksStatusRequestKind,
};
use std::sync::Arc;

pub async fn get_link_result_workflow<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResult, HolochainError> {
    let links = await!(get_links(
        context.clone(),
        link_args.entry_address.clone(),
        link_args.link_type.clone(),
        link_args.tag.clone(),
        link_args.options.timeout.clone()
    ))?;

    let (link_results, errors): (Vec<_>, Vec<_>) = links
        .iter()
        .map(|link| {
            get_latest_entry(
            context.block_on(
                get_entry_with_meta_workflow(&context, &link, &link_args.options.timeout).map(
                    |link_entry_result| {
                        match link_entry_result {
                            Ok(Some(EntryWithMetaAndHeader {
                                entry_with_meta: EntryWithMeta{entry: Entry::LinkAdd(link_data), ..},
                                headers,
                            })) => {
                                let headers = match link_args.options.headers {
                                    true => headers,
                                    false => Vec::new(),
                                };
                                Ok(LinksResult {
                                    address: link_data.link().target().clone(),
                                    headers,
                                    tag: link_data.link().tag().to_string(),
                        crud_link: link_entry.entry_with_meta.maybe_link_update_delete,
                                })
                            },
                            Ok(None) => {
                                Err(HolochainError::ErrorGeneric(
                                    format!("Could not get link entry for address stored in the EAV entry {}", link),
                                ))
                            }
                            Err(e) => {
                                Err(HolochainError::ErrorGeneric(
                                    format!("Error retrieveing link: {:?}", e),
                                ))
                            },
                            _ => {
                                Err(HolochainError::ErrorGeneric(
                                    format!("Unknown Error retrieveing link. Most likely EAV entry points to non-link entry type"),
                                ))
                            }
                        }
                    },
                ),
            )
        })
        .partition(Result::is_ok);

    if errors.is_empty() {
        Ok(GetLinksResult::new(
            link_results
                .into_iter()
                .map(|s| s.unwrap())
                .filter(|link_result| match link_args.options.status_request {
                    LinksStatusRequestKind::All => true,
                    LinksStatusRequestKind::Deleted => {
                        link_result.crud_status == CrudStatus::Deleted
                    }
                    LinksStatusRequestKind::Live => link_result.crud_status == CrudStatus::Live,
                })
                .collect(),
        ))
    } else {
        Err(HolochainError::ErrorGeneric(format!(
            "Could not get links: {:?}",
            errors
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
