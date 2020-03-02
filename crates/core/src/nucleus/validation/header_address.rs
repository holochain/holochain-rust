
use holochain_core_types::{chain_header::ChainHeader, entry::Entry};
use holochain_core_types::validation::{ValidationResult};

use holochain_persistence_api::cas::content::AddressableContent;

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn validate_header_address(entry: &Entry, header: &ChainHeader) -> ValidationResult {
    if entry.address() == *header.entry_address() {
        ValidationResult::Ok
    } else {
        ValidationResult::Fail("Wrong header for entry".to_string())
    }
}
