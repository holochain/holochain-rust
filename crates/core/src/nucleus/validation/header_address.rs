use crate::nucleus::validation::{ValidationError, ValidationResult};
use boolinator::Boolinator;
use holochain_core_types::{chain_header::ChainHeader, entry::Entry};

use holochain_persistence_api::cas::content::AddressableContent;

#[cfg(not(target_arch = "wasm32"))]
#[flame]
pub fn validate_header_address(entry: &Entry, header: &ChainHeader) -> ValidationResult {
    (entry.address() == *header.entry_address())
        .ok_or(ValidationError::Fail("Wrong header for entry".to_string()))
}
