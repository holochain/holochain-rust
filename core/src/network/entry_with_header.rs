use crate::agent::find_chain_header;
use holochain_core_types::{
    cas::content::Address, chain_header::ChainHeader, entry::Entry, error::HolochainError,
};
use std::convert::TryInto;
use crate::state::State;

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

fn fetch_entry_from_cas(
    address: &Address,
    state: &State,
) -> Result<Entry, HolochainError> {
    let json = state
        .agent()
        .chain_store()
        .content_storage()
        .read()?
        .fetch(address)?
        .ok_or("Entry not found".to_string())?;
    let s: Entry = json.try_into()?;
    Ok(s.into())
}

pub fn fetch_entry_with_header(
    address: &Address,
    state: &State,
) -> Result<EntryWithHeader, HolochainError> {
    let entry = fetch_entry_from_cas(address, state)?;

    let header =
        find_chain_header(&entry, state).ok_or("No header found for entry".to_string())?;

    Ok(EntryWithHeader::new(entry, header))
}
