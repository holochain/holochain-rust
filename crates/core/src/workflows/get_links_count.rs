use crate::{
    context::Context,
    network::{
        actions::query::{query, QueryMethod},
        query::{GetLinksNetworkQuery, GetLinksNetworkResult, NetworkQueryResult},
    },
    NEW_RELIC_LICENSE_KEY,
};
use holochain_wasm_types::ZomeApiResult;
use holochain_core_types::error::HolochainError;
use holochain_wasm_types::get_links::{GetLinksArgs, GetLinksResultCount};
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn get_link_result_count_workflow<'a>(
    context: Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResultCount, HolochainError> {
    let method = QueryMethod::Link(link_args.clone(), GetLinksNetworkQuery::Count);
    let response = query(context.clone(), method, link_args.options.timeout.clone()).await?;

    let links_result = match response {
        NetworkQueryResult::Links(link_result, _, _) => Ok(link_result),
        NetworkQueryResult::Entry(_) => Err(HolochainError::ErrorGeneric(
            "Could not get link".to_string(),
        )),
    }?;

    let links_count = match links_result {
        GetLinksNetworkResult::Count(count) => Ok(count),
        _ => Err(HolochainError::ErrorGeneric(
            "Getting wrong type of GetLinks".to_string(),
        )),
    }?;

    Ok(GetLinksResultCount { count: links_count })
}

pub fn invoke_get_links_count(context: Arc<Context>, link_args: GetLinksArgs) -> ZomeApiResult {
    context.block_on(get_link_result_count_workflow(context, &link_args))
}
