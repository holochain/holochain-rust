use holochain_core_types::{json::*, validation::ValidationData};

#[derive(Deserialize, Debug, Serialize)]
pub struct EntryValidationArgs {
    pub entry_type: String,
    pub entry: String,
    pub validation_data: ValidationData,
}

impl From<EntryValidationArgs> for JsonString {
    fn from(v: EntryValidationArgs) -> Self {
        default_to_json(v)
    }
}
