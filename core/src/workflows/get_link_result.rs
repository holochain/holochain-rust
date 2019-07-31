use crate::{
    context::Context,
    action::RespondGetPayload,
    network::{
        actions::get_entry::{GetMethod,get_entry},
        query::{GetLinksNetworkQuery, GetLinksNetworkResult,GetLinksQueryConfiguration},
    },
};

use holochain_core_types::{error::HolochainError,time::Timeout};
use holochain_wasm_utils::api_serialization::get_links::{GetLinksArgs, GetLinksResult, LinksResult};
use std::sync::Arc;

pub async fn get_link_result_workflow<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResult, HolochainError> {
    let config = GetLinksQueryConfiguration
    {
        headers : link_args.options.headers
    };
    let method = GetMethod::Link(link_args.clone(),GetLinksNetworkQuery::Links(config));
    let response = await!(get_entry(
        context.clone(),
        method,
        Timeout::default()
    ))?;

    let links_result = match response
    {
        RespondGetPayload::Links((query,_,_)) => Ok(query),
        _ => Err((HolochainError::ErrorGeneric("Wrong type for response type Entry".to_string())))
    }?;

    match links_result
    {
        GetLinksNetworkResult::Links(links) =>
        {
            let get_links_result = links
            .into_iter()
            .map(|get_entry_crud| LinksResult {
                address: get_entry_crud.target.clone(),
                headers: get_entry_crud.headers.unwrap_or_default(),
                status: get_entry_crud.crud_status.clone(),
                tag: get_entry_crud.tag.clone()
            })
            .collect::<Vec<LinksResult>>();

            Ok(GetLinksResult::new(get_links_result))

        },
        _ => Err(HolochainError::ErrorGeneric("Could not get links".to_string()))
    }
}


