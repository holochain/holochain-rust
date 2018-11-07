use holochain_core_types::{error::HolochainError, json::*, validation::ValidationData};

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct EntryValidationArgs {
    pub entry_type: String,
    pub entry: String,
    pub validation_data: ValidationData,
}
