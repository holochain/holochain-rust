use holochain_core_types::{
    entry::Entry,
    link::Link,
    validation::{EntryValidationData, LinkValidationData},
};

use lib3h_persistence_api::{
    error::PersistenceError,
    json::*,
};


#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct EntryValidationArgs {
    pub validation_data: EntryValidationData<Entry>,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, PartialEq, Clone)]
pub enum LinkDirection {
    To,
    From,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct LinkValidationPackageArgs {
    pub entry_type: String,
    pub link_type: String,
    pub direction: LinkDirection,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct LinkValidationArgs {
    pub entry_type: String,
    pub link: Link,
    pub direction: LinkDirection,
    pub validation_data: LinkValidationData,
}
