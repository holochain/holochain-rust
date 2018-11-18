use holochain_core_types::{error::HolochainError, json::*, validation::ValidationData};

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct EntryValidationArgs {
    pub entry_type: String,
    pub entry: String,
    pub validation_data: ValidationData,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, PartialEq, Clone)]
pub enum LinkDirection {
    To,
    From,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct LinkValidationPackageArgs {
    pub entry_type: String,
    pub tag: String,
    pub direction: LinkDirection
}