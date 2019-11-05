use crate::{
    agent::find_chain_header,
    state::State,
};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry, error::HolochainError};
use holochain_persistence_api::cas::content::Address;
use std::convert::TryInto;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryWithHeader {
    pub entry: Entry,
    pub header: ChainHeader,
}

impl EntryWithHeader {
    pub fn new(entry: Entry, header: ChainHeader) -> EntryWithHeader {
        EntryWithHeader { entry, header }
    }
}

fn fetch_entry_from_cas(address: &Address, state: &State) -> Result<Entry, HolochainError> {
    let json = state
        .agent()
        .chain_store()
        .content_storage()
        .read()?
        .fetch(address)?
        .ok_or_else(|| HolochainError::from("Entry not found"))?;
    let s: Entry = json.try_into()?;
    Ok(s)
}

pub fn fetch_entry_with_header(
    address: &Address,
    state: &State,
) -> Result<EntryWithHeader, HolochainError> {
    let entry = fetch_entry_from_cas(address, state)?;

    let header = find_chain_header(&entry, &state)
        .ok_or_else(|| HolochainError::from("No header found for entry"))?;

    Ok(EntryWithHeader::new(entry, header))
}
