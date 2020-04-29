use crate::{
    context::Context,
    network::{
        actions::query::{crud_status_from_link_args, query, QueryMethod},
        handler::query::get_links,
        query::{
            GetLinksNetworkQuery, GetLinksNetworkResult, GetLinksQueryConfiguration,
            NetworkQueryResult,
        },
    },
};

use holochain_core_types::error::HolochainError;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use holochain_wasm_utils::api_serialization::get_links::{
    GetLinksArgs, GetLinksResult, LinksResult,
};
use lib3h_protocol::types::EntryHash;
use std::sync::Arc;

// check to see if we are an authority on the DHT for this base, if so no need
// to go out to request this from anybody else.  We know for sure that we are an authority
// for anything that is linked on our own agent hash.
pub fn am_i_dht_authority_for_base<'a>(context: &'a Arc<Context>, base: &Address) -> bool {
    let me: Address = context.agent_id.address();
    if *base == me {
        return true;
    }
    let state = context
        .state()
        .expect("No state present when trying to respond with gossip list");
    state
        .dht()
        .get_holding_map()
        .contains_entry(&EntryHash::from(base))
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn get_link_result_workflow<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResult, HolochainError> {
    let config = GetLinksQueryConfiguration {
        headers: link_args.options.headers,
        pagination: link_args.options.pagination.clone(),
        sort_order: link_args.options.sort_order.clone(),
    };
    let method = QueryMethod::Link(
        link_args.clone(),
        GetLinksNetworkQuery::Links(config.clone()),
    );
    let links = if am_i_dht_authority_for_base(context, &link_args.entry_address) {
        // get the results from the local DHT
        let links = get_links(
            context,
            link_args.entry_address.clone(),
            link_args.link_type.clone(),
            link_args.tag.clone(),
            crud_status_from_link_args(&link_args),
            config,
        )?;
        links
    } else {
        let response = query(context.clone(), method, link_args.options.timeout.clone()).await?;
        let links_result = match response {
            NetworkQueryResult::Links(query, _, _) => Ok(query),
            _ => Err(HolochainError::ErrorGeneric(
                "Wrong type for response type Entry".to_string(),
            )),
        }?;
        match links_result {
            GetLinksNetworkResult::Links(links) => links,
            _ => {
                return Err(HolochainError::ErrorGeneric(
                    "Could not get links".to_string(),
                ))
            }
        }
    };

    let get_links_result = links
        .into_iter()
        .map(|get_entry_crud| LinksResult {
            address: get_entry_crud.target.clone(),
            headers: get_entry_crud.headers.unwrap_or_default(),
            status: get_entry_crud.crud_status,
            tag: get_entry_crud.tag.clone(),
        })
        .collect::<Vec<LinksResult>>();

    Ok(GetLinksResult::new(get_links_result))
}
