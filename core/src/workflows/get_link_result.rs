use crate::{context::Context, network::actions::get_links::get_links,
workflows::get_entry_result::get_entry_with_meta_workflow};

use holochain_core_types::{
    error::HolochainError
};
use holochain_wasm_utils::api_serialization::get_links::{GetLinksResult,GetLinksArgs,LinksStatusRequestKind};
use std::sync::Arc;




pub async fn get_link_result_workflow<'a>(
    context: &'a Arc<Context>,
    link_args: &'a GetLinksArgs,
) -> Result<GetLinksResult, HolochainError> 
{
    //get entry from dht
    let entry_with_workflow = await!(get_entry_with_meta_workflow(&context,&link_args.entry_address,&link_args.options.timeout))?;
    
    // if option for headers is selected get headers from entry otherwise return empty array
    let headers = if link_args.options.headers
    {
        entry_with_workflow
        .ok_or(HolochainError::ErrorGeneric("Could not get entry".to_string()))?
        .headers
    }
    else
    {
        Vec::new()
    };
    
    // will tackle this when it is some to work with crud_status, refraining from using return because not idiomatic rust
    if link_args.options.status_request != LinksStatusRequestKind::Live
    {
        Err(HolochainError::ErrorGeneric("Status rather than live not implemented".to_string()))
    }
    else
    {
        Ok(())
    }?;
    //get links
    let links = await!(get_links(context.clone(),link_args.entry_address.clone(),link_args.tag.clone(),link_args.options.timeout.clone()))?;
    Ok(GetLinksResult::new(links,headers))
}