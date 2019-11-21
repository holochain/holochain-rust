use crate::{
    agent::find_chain_header,
    content_store::GetContent,
    state::{State, StateWrapper},
};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry, error::HolochainError};
use holochain_persistence_api::cas::content::Address;
use std::convert::TryInto;

/// A `ChainPair` cannot be constructed unless the entry address in the
/// `ChainHeader` that is within the `ChainPair` is the same as the address
/// of the `Entry` that is also within the `ChainPair`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChainPair(ChainHeader, Entry);

impl ChainPair {
    pub fn new(header: ChainHeader, entry: Entry) -> Result<ChainPair, HolochainError> {
        let header_entry = header.entry();
        let header_entry_address = header_entry.address();
        let entry_address = entry.address();
        if header_entry_address == entry_address {
            Ok(ChainPair(header, entry))
        } else {
            Err(HolochainError::HeaderEntryMismatch(
                "Tried to create a ChainPair, but got a mismatch with the header's entry address {} and the entry's address {}.",
                header_entry_address,
                entry_address,
            ))
        }
    }

    pub fn header(&self) -> ChainHeader {
        self.0.clone()
    }

    pub fn entry(&self) -> Entry {
        self.1.clone()
    }

    pub fn fetch_chain_pair(
        address: &Address,
        state: &State
    ) -> Result<ChainPair, HolochainError> {
        let entry = state
            .agent()
            .chain_store()
            .get(address)?
            .ok_or_else(|| HolochainError::from("Entry not found"))?;

        let header =
            find_chain_header(&entry, &StateWrapper::from(state.clone())).ok_or_else(|| {
                HolochainError::from(
                    "No header found for the address {}. Entry:\n{:#?}\n",
                    address, entry
                )
            })?;
        ChainPair::new(entry, header)
    }
}
