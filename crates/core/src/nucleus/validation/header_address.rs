use crate::nucleus::validation::{ValidationError, ValidationResult};
use boolinator::Boolinator;
use holochain_core_types::{chain_header::ChainHeader, entry::Entry};

use holochain_persistence_api::cas::content::AddressableContent;

//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn validate_header_address(entry: &Entry, header: &ChainHeader) -> ValidationResult {
    (entry.address() == *header.entry_address())
        .ok_or(ValidationError::Fail("Wrong header for entry".to_string()))
}
