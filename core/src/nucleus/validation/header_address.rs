use crate::nucleus::validation::{ValidationError, ValidationResult};
use boolinator::Boolinator;
use holochain_core_types::{
    cas::content::AddressableContent, chain_header::ChainHeader, entry::Entry,
};

pub fn validate_header_address(entry: &Entry, header: &ChainHeader) -> ValidationResult {
    (entry.address() == *header.entry_address())
        .ok_or(ValidationError::Fail("Wrong header for entry".to_string()))
}
