use crate::{
    agent::find_chain_header,
    content_store::GetContent,
    state::{State, StateWrapper},
};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry, error::HolochainError};
use holochain_persistence_api::cas::content::Address;

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

pub fn fetch_entry_with_header(
    address: &Address,
    state: &State,
) -> Result<EntryWithHeader, HolochainError> {
    let entry = state
        .agent()
        .chain_store()
        .get(address)?
        .ok_or_else(|| HolochainError::from("Entry not found"))?;

    let header = find_chain_header(&entry, &StateWrapper::from(state.clone()))
        .ok_or_else(|| HolochainError::from("No header found for entry"))?;

    Ok(EntryWithHeader::new(entry, header))
}
