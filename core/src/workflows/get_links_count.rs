use crate::{context::Context, network::{actions::get_links::get_links,query::{GetLinksNetworkQuery,GetLinksNetworkResult}}};

use holochain_core_types::error::HolochainError;
use holochain_wasm_utils::api_serialization::get_links::{GetLinksArgs, GetLinksResultCount};
use std::sync::Arc;

pub async fn get_link_result_count_workflow<'a>(
    context: Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResultCount, HolochainError> {
    let links_result = await!(get_links(
        context,
        link_args,
        GetLinksNetworkQuery::Count
    ))?;

    let links_count = match links_result
    {
        GetLinksNetworkResult::Count(count) => Ok(count),
        _ => Err(HolochainError::ErrorGeneric("Getting wrong type of GetLinks".to_string()))
    }?;

    Ok(GetLinksResultCount { count: links_count })
}
