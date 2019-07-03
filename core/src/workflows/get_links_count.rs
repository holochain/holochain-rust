use crate::{context::Context, network::actions::get_links_count::{get_links_count,get_links_count_by_tag}};

use holochain_core_types::error::HolochainError;
use holochain_wasm_utils::api_serialization::get_links::{GetLinksBy, GetLinksResultCount};
use std::sync::Arc;

pub async fn get_link_result_count_workflow<'a>(
    context: Arc<Context>,
    link_args: &'a GetLinksBy,
) -> Result<GetLinksResultCount, HolochainError> {
    let links_count = match link_args {
        GetLinksBy::GetLinksArgs(link_args) => await!(get_links_count(
            context,
            link_args.entry_address.clone(),
            link_args.link_type.clone(),
            link_args.tag.clone(),
            link_args.options.timeout.clone(),
            link_args.options.status_request.clone()
        ))?,
        GetLinksBy::Tag(tag, options) =>
        {
            await!(get_links_count_by_tag(context,tag.to_string(),options.timeout.clone(),options.status_request.clone()))?
        }
    };
    //get links based on status request, all for everything, deleted for deleted links and live for active links

    Ok(GetLinksResultCount { count: links_count })
}
