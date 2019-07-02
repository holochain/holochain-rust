use crate::{context::Context, network::actions::get_links_count::get_links_count};

use holochain_core_types::error::HolochainError;
use holochain_wasm_utils::api_serialization::get_links::{GetLinksArgs, GetLinksResultCount};
use std::sync::Arc;

pub async fn get_link_result_count_workflow<'a>(
    context: Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResultCount, HolochainError> {
    let links_count = await!(get_links_count(
        context,
        link_args.entry_address.clone(),
        link_args.link_type.clone(),
        link_args.tag.clone(),
        link_args.options.timeout.clone(),
        link_args.options.status_request.clone()
    ))?;
    //get links based on status request, all for everything, deleted for deleted links and live for active links

    Ok(GetLinksResultCount { count: links_count })
}
