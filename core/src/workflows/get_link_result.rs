use crate::{
    context::Context,
    network::{
        actions::get_links::get_links,
        query::{GetLinksNetworkQuery, GetLinksNetworkResult,GetLinksQueryConfiguration},
    },
};

use holochain_core_types::error::HolochainError;
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
    let links_result = await!(get_links(
        context.clone(),
        link_args,
        GetLinksNetworkQuery::Links(config)
    ))?;

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


