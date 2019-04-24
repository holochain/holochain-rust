use crate::{
    context::Context, network::actions::get_links::get_links,
    workflows::get_entry_result::get_entry_result_workflow,
};

use futures_util::future::FutureExt;
use holochain_core_types::error::HolochainError;
use holochain_wasm_utils::api_serialization::{get_links::{
    GetLinksArgs, GetLinksResult, LinksResult, LinksStatusRequestKind,
},
get_entry::{GetEntryArgs,GetEntryOptions}};
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
        .into_iter()
        .map(|link| {
            let entry_args = GetEntryArgs
            {
                address : link,
                options : GetEntryOptions
                {
                    status_request : link_args.options.link_status_request.clone(),
                    headers : link_args.options.headers.clone(),
                    timeout : link_args.options.timeout.clone(),
                    ..Default::default()
                }
            
            };
            context
                .block_on(get_entry_result_workflow(&context, &entry_args))
        })
        .partition(Result::is_ok);

    if errors.is_empty() {
        Ok(GetLinksResult::new(
            link_results.into_iter().map(|s|LinksResult{ link : s.unwrap()}).collect(),
        ))
    } else {
        Err(HolochainError::ErrorGeneric(
            "Could not get links".to_string(),
        ))
    }
}
