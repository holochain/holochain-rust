use crate::{
    agent::find_chain_header,
    content_store::GetContent,
    state::{State, StateWrapper},
};
use holochain_core_types::{chain_header::{ChainHeader, test_chain_header}, entry::{Entry, test_entry}, error::HolochainError};
use holochain_persistence_api::cas::content::{Address, AddressableContent};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryWithHeader {
    pub entry: Entry,
    pub header: ChainHeader,
}

impl EntryWithHeader {
    pub fn new(entry: Entry, header: ChainHeader) -> EntryWithHeader {
        EntryWithHeader { entry, header }
    }

    pub fn try_from_entry_and_header(
        entry: Entry,
        header: ChainHeader,
    ) -> Result<EntryWithHeader, HolochainError> {
        if entry.address() != *header.entry_address() {
            Err(HolochainError::ValidationFailed(String::from(
                "Entry/Header mismatch",
            )))
        } else {
            Ok(EntryWithHeader::new(entry, header))
        }
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


#[cfg_attr(tarpaulin, skip)]
pub fn test_entry_with_header() -> EntryWithHeader {
    let entry = test_entry();
    let header = test_chain_header();
    EntryWithHeader {
        entry,
        header,
    }
}
