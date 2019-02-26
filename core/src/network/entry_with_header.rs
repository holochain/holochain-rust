use crate::{agent::find_chain_header, context::Context};
use holochain_core_types::{
    cas::content::Address, chain_header::ChainHeader, entry::Entry, error::HolochainError,
};
use std::{convert::TryInto, sync::Arc};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryWithHeader {
    pub entry: Entry,
    pub header: ChainHeader,
}

impl EntryWithHeader {
    pub fn new(entry: Entry, header: ChainHeader) -> EntryWithHeader {
        EntryWithHeader { entry, header }
    }

    pub fn set_entry(&mut self, entry : Entry)
    {
        self.entry = entry
    }
}

fn fetch_entry_from_cas(
    address: &Address,
    context: &Arc<Context>,
) -> Result<Entry, HolochainError> {
    let json = context
        .dht_storage
        .read()?
        .fetch(address)?
        .ok_or("Entry not found".to_string())?;
    let s: Entry = json.try_into()?;
    Ok(s.into())
}

pub fn fetch_entry_with_header(
    address: &Address,
    context: &Arc<Context>,
) -> Result<EntryWithHeader, HolochainError> {
    let entry = fetch_entry_from_cas(address, &context)?;
    let header =
        find_chain_header(&entry, &context).ok_or("No header found for entry".to_string())?;

    Ok(EntryWithHeader::new(entry, header))
}
