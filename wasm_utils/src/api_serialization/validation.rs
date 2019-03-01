use holochain_core_types::{
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    json::*,
    link::Link,
    validation::ValidationData,
};

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct EntryValidationArgs {
    pub entry_type: EntryType,
    pub entry: Entry,
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
    pub direction: LinkDirection,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct LinkValidationArgs {
    pub entry_type: String,
    pub link: Link,
    pub direction: LinkDirection,
    pub validation_data: ValidationData,
}
