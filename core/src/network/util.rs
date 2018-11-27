use crate::{
    agent::chain_header,
    context::Context,
};
use holochain_core_types::{
    cas::content::Address,
    chain_header::ChainHeader,
    entry::{Entry, SerializedEntry},
    error::HolochainError,
};
use std::{convert::TryInto, sync::Arc};

#[derive(Serialize, Deserialize)]
pub struct EntryWithHeader {
    pub entry: SerializedEntry,
    pub header: ChainHeader,
}

impl From<(Entry, ChainHeader)> for EntryWithHeader {
    fn from((entry, header): (Entry, ChainHeader)) -> EntryWithHeader{
        EntryWithHeader {
            entry: entry.serialize(), header,
        }
    }
}

pub fn entry_from_cas(address: &Address, context: &Arc<Context>) -> Result<Entry, HolochainError> {
    let json = context
        .file_storage
        .read()?
        .fetch(address)?
        .ok_or("Entry not found".to_string())?;
    let s: SerializedEntry = json.try_into()?;
    Ok(s.into())
}

pub fn entry_with_header(address: &Address, context: &Arc<Context>) -> Result<(Entry, ChainHeader), HolochainError> {
    let entry = entry_from_cas(address, &context)?;
    let header = chain_header(&entry, &context)
        .ok_or("No header found for entry".to_string())?;

    Ok((entry, header))
}