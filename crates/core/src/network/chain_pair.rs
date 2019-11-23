use crate::{
    agent::find_chain_header,
    content_store::GetContent,
    state::{State, StateWrapper},
};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry, error::HolochainError};
use holochain_persistence_api::cas::content::{Address, AddressableContent};

/// A `ChainPair` cannot be constructed unless the entry address in the
/// `ChainHeader` that is within the `ChainPair` is the same as the address
/// of the `Entry` that is also within the `ChainPair`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChainPair(ChainHeader, Entry);

impl ChainPair {
    // It seems best to not have a new method, since stylistically it is expected to return Self, whereas constructing could fail.
    pub fn try_from_header_and_entry(
        header: ChainHeader,
        entry: Entry,
    ) -> Result<ChainPair, HolochainError> {
        let header_entry_address = *header.entry_address();
        let entry_address = entry.address();
        if header_entry_address == entry_address {
            Ok(ChainPair(header, entry))
        } else {
            let error_msg = format!("Tried to create a ChainPair, but got a mismatch with the header's entry address and the entry's address. Header:\n{:#?}\nEntry:{:#?}", header, entry);
            Err(HolochainError::HeaderEntryMismatch(
                error_msg, header_entry_address, entry_address
            ))
        }
    }

    // Convenience function for returning a custom error in the context of validation.
    pub fn try_validate_from_entry_and_header(
        entry: Entry,
        header: ChainHeader,
        entry_aspect: EntryAspect,
    ) -> Result<ChainPair, HolochainError> {
        ChainPair::try_from_header_and_entry(header, entry)
            .map_err(|e| HolochainError::ValidationFailed(String::from(e)))
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
                let error_msg = format!(
                    "No header found for the address:\n{}\nEntry:\n{:#?}\n",
                    address, entry
                );
                HolochainError::from(error_msg)
            })?;
        ChainPair::try_from_header_and_entry(header, entry)
    }
}
