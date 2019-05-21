use crate::{
    context::Context, network::actions::get_links::get_links,
    workflows::get_entry_result::get_entry_with_meta_workflow,
};

use futures_util::future::FutureExt;
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
    // will tackle this when it is some to work with crud_status, refraining from using return because not idiomatic rust
    if link_args.options.status_request != LinksStatusRequestKind::Live {
        Err(HolochainError::ErrorGeneric(
            "Status rather than live not implemented".to_string(),
        ))
    } else {
        Ok(())
    }?;
    //get links
    let links = await!(get_links(
        context.clone(),
        link_args.entry_address.clone(),
        link_args.tag.clone(),
        link_args.options.timeout.clone()
    ))?;

    let (link_results, errors): (Vec<_>, Vec<_>) = links
        .iter()
        .map(|link| {
            //we should probably replace this with get_entry_result_workflow, it does all the work needed
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
            link_results.into_iter().map(|s| s.unwrap()).collect(),
        ))
    } else {
        Err(HolochainError::ErrorGeneric(format!(
            "Could not get links: {:?}",
            errors
        )))
    }
}
