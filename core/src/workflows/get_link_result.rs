use crate::{
    context::Context, network::actions::get_links::get_links,
    workflows::get_entry_result::get_entry_with_meta_workflow,
};

use futures_util::future::FutureExt;
use holochain_core_types::{entry::EntryWithMetaAndHeader, error::HolochainError};
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
                        if let Ok(Some(EntryWithMetaAndHeader {
                            entry_with_meta: _,
                            headers,
                        })) = link_entry_result
                        {
                            let headers = match link_args.options.headers {
                                true => headers,
                                false => Vec::new(),
                            };
                            Ok(LinksResult {
                                address: link.clone(),
                                headers,
                            })
                        } else {
                            Err(HolochainError::ErrorGeneric(
                                "Could not get links".to_string(),
                            ))
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
        Err(HolochainError::ErrorGeneric(
            "Could not get links".to_string(),
        ))
    }
}
