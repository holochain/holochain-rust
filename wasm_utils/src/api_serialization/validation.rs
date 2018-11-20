use holochain_core_types::{error::HolochainError, json::*, validation::ValidationData};
use holochain_core_types::entry::entry_type::EntryType;
use holochain_core_types::entry::Entry;

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct EntryValidationArgs {
    pub entry_type: EntryType,
    pub entry: Entry,
    pub validation_data: ValidationData,
}
