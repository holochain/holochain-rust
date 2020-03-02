use crate::{
    agent::find_chain_header,
    content_store::GetContent,
    state::{State, StateWrapper},

};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry, error::HolochainError, validation::ValidationResult};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use holochain_json_api::json::JsonString;
use holochain_json_api::error::JsonError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct EntryWithHeader {
    pub entry: Entry,
    pub header: ChainHeader,
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl EntryWithHeader {
    pub fn new(entry: Entry, header: ChainHeader) -> EntryWithHeader {
        EntryWithHeader { entry, header }
    }

    pub fn try_from_entry_and_header(
        entry: Entry,
        header: ChainHeader,
    ) -> Result<EntryWithHeader, HolochainError> {
        if entry.address() != *header.entry_address() {
            Err(HolochainError::ValidationFailed(ValidationResult::Fail(
                "Entry/Header mismatch".into()
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
