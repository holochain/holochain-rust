use crate::{
    context::Context,
    dht::actions::hold::hold_entry,
    network::entry_with_header::EntryWithHeader,
};

use holochain_core_types::{
    error::HolochainError,
    chain_header::ChainHeader,
    entry::Entry,
};

use holochain_persistence_api::cas::content::AddressableContent;

use std::sync::Arc;

pub async fn hold_header_workflow(
    chain_header: &ChainHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {

    // 1. No need to validate. Store header in local DHT shard
    // create an entry_with_header to leverage existing functionality. A headers header is itself.
    let chain_header_entry = Entry::ChainHeader(chain_header.clone());
    let entry_with_header = EntryWithHeader::new(chain_header_entry.clone(), chain_header.clone());
    await!(hold_entry(&entry_with_header, context.clone()))?;

    context.log(format!(
        "debug/workflow/hold_header: HOLDING: {}",
        chain_header_entry.address()
    ));

    Ok(())
}
