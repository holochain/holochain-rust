use holochain_core_types::{
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    json::*,
    validation::ValidationData,
};

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct EntryValidationArgs {
    pub entry_type: EntryType,
    pub entry: Entry,
    pub validation_data: ValidationData,
}
