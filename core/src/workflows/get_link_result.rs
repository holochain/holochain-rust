use crate::{
    context::Context, network::actions::get_links::get_links,
    workflows::get_entry_result::get_entry_with_meta_workflow,
};

use futures_util::future::FutureExt;
use holochain_core_types::{cas::content::Address, entry::Entry, error::HolochainError};
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
    let links = await!(get_link_caches(context, link_args))?;

    let (link_results, errors): (Vec<_>, Vec<_>) = links
        .into_iter()
        .map(|link| {
            //we should probably replace this with get_entry_result_workflow, it does all the work needed
            context
                .block_on(
                    get_entry_with_meta_workflow(&context, &link.0, &link_args.options.timeout).map(
                        |link_entry_result| {
                            link_entry_result
                                .map(|link_entry_option| {
                                    link_entry_option.map(|link_entry| {
                                        let headers = if link_args.options.headers {
                                            link_entry.headers
                                        } else {
                                            Vec::new()
                                        };
                                        Ok(LinksResult {
                                            address: link.1.clone().clone(),
                                            headers,
                                        })
                                    })
                                })
                                .unwrap_or(None)
                        },
                    ),
                )
                .unwrap_or(Err(HolochainError::ErrorGeneric(
                    "Could not get links".to_string(),
                )))
        })
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
) -> Result<Vec<(Address,Address)>, HolochainError> {
    let links_caches = await!(get_links(
        context.clone(),
        link_args.entry_address.clone(),
        link_args.tag.clone(),
        link_args.options.timeout.clone()
    ))?;

    let (links_result, get_links_error): (Vec<_>, Vec<_>) = links_caches
        .iter()
        .map(|s| {
            let entry_with_header = context.block_on(get_entry_with_meta_workflow(
                &context.clone(),
                &s.clone(),
                &link_args.options.timeout.clone(),
            ));
            entry_with_header
                .map(|link_entry_result| {
                    link_entry_result.map(|link_entry| match link_entry.entry_with_meta.entry {
                        Entry::LinkAdd(link) => Ok((s.clone(),link.link().target().clone())),
                        _ => Err(HolochainError::ErrorGeneric(
                            "expected entry of type link".to_string(),
                        )),
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
        Err(HolochainError::ErrorGeneric(
            "Could not get links".to_string(),
        ))
    }
}
