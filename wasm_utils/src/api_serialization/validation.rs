use holochain_core_types::validation::ValidationData;

#[derive(Deserialize, Debug, Serialize)]
pub struct EntryValidationArgs {
    pub entry_type: String,
    pub entry: String,
    pub validation_data: ValidationData,
}
