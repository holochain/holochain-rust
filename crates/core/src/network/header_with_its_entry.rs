use crate::{
    agent::find_chain_header,
    content_store::GetContent,
    state::{State, StateWrapper},
};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry, error::HolochainError};
use holochain_persistence_api::cas::content::{Address, AddressableContent};

/// A `EntryHeaderPair` cannot be constructed unless the entry address in the
/// `ChainHeader` that is within the `EntryHeaderPair` is the same as the address
/// of the `Entry` that is also within the `EntryHeaderPair`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryHeaderPair(ChainHeader, Entry);

impl EntryHeaderPair {
    // It seems best to not have a new method, since stylistically it is expected to return Self, whereas constructing could fail.
    pub fn try_from_header_and_entry(
        header: ChainHeader,
        entry: Entry,
    ) -> Result<EntryHeaderPair, HolochainError> {
        let header_entry_address = header.entry_address();
        let entry_address = entry.address();
        if header_entry_address.clone() == entry_address {
            Ok(EntryHeaderPair(header, entry))
        } else {
            let basic_error_msg = "Tried to create a EntryHeaderPair, but got a
            mismatch with the header's entry address and the entry's
            address.";
            let error_msg = format!(
                "{} See the debug log output for data for the header and entry.",
                basic_error_msg
            );
            debug!(
                "{}\nHeader:\n{:#?}\nEntry:{:#?}\nentry in header (i.e. header.entry()=\n",
                basic_error_msg, header, entry
            );
            Err(HolochainError::HeaderEntryMismatch(
                error_msg,
                header_entry_address.clone(),
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

    pub fn fetch_entry_header_pair(
        address: &Address,
        state: &State,
    ) -> Result<EntryHeaderPair, HolochainError> {
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
        EntryHeaderPair::try_from_header_and_entry(header, entry)
    }
}
