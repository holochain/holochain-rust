use crate::{
    context::Context,
    network::{
        actions::query::{query, QueryMethod},
        query::{
            GetLinkData, GetLinksNetworkQuery, GetLinksNetworkResult, GetLinksQueryConfiguration,
            NetworkQueryResult,
        },
    },
    NEW_RELIC_LICENSE_KEY,
    workflows::get_entry_result::get_entry_result_workflow,
};
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::get_entry::{
    GetEntryArgs, GetEntryOptions, GetEntryResultType,
};

use holochain_core_types::{
    crud_status::CrudStatus, entry::Entry, error::HolochainError,
    network::query::GetLinkFromRemoteData,
};
use holochain_wasm_utils::api_serialization::get_links::{
    GetLinksArgs, GetLinksResult, LinksResult,
};
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn get_link_result_workflow<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResult, HolochainError> {
    let config = GetLinksQueryConfiguration {
        headers: link_args.options.headers,
    };
    let method = QueryMethod::Link(link_args.clone(), GetLinksNetworkQuery::Links(config));
    let response = query(context.clone(), method, link_args.options.timeout.clone()).await?;

    let links_result = match response {
        NetworkQueryResult::Links(query, _, _) => Ok(query),
        _ => Err(HolochainError::ErrorGeneric(
            "Wrong type for response type Entry".to_string(),
        )),
    }?;

    match links_result {
        GetLinksNetworkResult::Links(links) => {
            links
                .into_iter()
                .map(
                    |GetLinkFromRemoteData {
                         link_add_address,
                         tag,
                         crud_status,
                     }| {
                        // make DHT calls to get the entries for the links
                        (
                            get_link_data_from_link_addresses(
                                context,
                                &link_add_address,
                                &tag,
                                link_args.options.headers,
                            ),
                            crud_status,
                        )
                    },
                )
                .map(|(maybe_get_entry_result, crud_status)| {
                    maybe_get_entry_result.map(|get_entry_result| LinksResult {
                        address: get_entry_result.target.clone(),
                        headers: get_entry_result.headers.unwrap_or_default(),
                        status: crud_status,
                        tag: get_entry_result.tag.clone(),
                    })
                })
                .collect::<Result<Vec<LinksResult>, HolochainError>>()
                .map(|get_links_result| GetLinksResult::new(get_links_result))
        }
        _ => Err(HolochainError::ErrorGeneric(
            "Could not get links".to_string(),
        )),
    }
}

// given the address of a link_add/link_remove entry, build a GetLinkData struct by retrieving the data from the DHT
pub fn get_link_data_from_link_addresses(
    context: &Arc<Context>,
    link_add_address: &Address,
    tag: &String,
    include_headers: bool,
) -> Result<GetLinkData, HolochainError> {
    let get_link_add_entry_args = GetEntryArgs {
        address: link_add_address.clone(),
        options: GetEntryOptions {
            headers: include_headers,
            ..Default::default()
        },
    };
    context
        .block_on(get_entry_result_workflow(
            &context.clone(),
            &get_link_add_entry_args,
        ))
        .map(|get_entry_result| match get_entry_result.result {
            GetEntryResultType::Single(entry_with_meta_and_headers) => {
                let maybe_entry_headers = if include_headers {
                    Some(entry_with_meta_and_headers.headers)
                } else {
                    None
                };
                let crud = entry_with_meta_and_headers
                    .meta
                    .map(|m| m.crud_status)
                    .unwrap_or(CrudStatus::Live);
                entry_with_meta_and_headers
                    .entry
                    .map(|single_entry| match single_entry {
                        Entry::LinkAdd(link_add) => Ok(GetLinkData::new(
                            link_add_address.clone(),
                            crud,
                            link_add.link().target().clone(),
                            tag.clone(),
                            maybe_entry_headers,
                        )),
                        Entry::LinkRemove(link_remove) => Ok(GetLinkData::new(
                            link_add_address.clone(),
                            crud,
                            link_remove.0.link().target().clone(),
                            tag.clone(),
                            maybe_entry_headers,
                        )),
                        _ => Err(HolochainError::ErrorGeneric(
                            format!("Wrong entry type for Link content. Expected LinkAdd or LinkRemove, got: {:?}", single_entry),
                        )),
                    })
                    .unwrap_or_else(|| {
                        Err(HolochainError::ErrorGeneric(format!(
                            "Could not find Entries for Address: {}, tag: {}",
                            link_add_address.clone(),
                            tag.clone()
                        )))
                    })
            }
            _ => Err(HolochainError::ErrorGeneric(
                "Single Entry required for Get Entry".to_string(),
            )),
        })
        .unwrap_or_else(|e| {
            Err(HolochainError::ErrorGeneric(format!(
                "Could not get entry for Link Data {:?}",
                e
            )))
        })
}
