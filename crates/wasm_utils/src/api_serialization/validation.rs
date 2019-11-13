use holochain_core_types::{
    agent::AgentId,
    entry::Entry,
    link::Link,
    validation::{EntryValidationData, LinkValidationData},
};

use holochain_json_api::{error::JsonError, json::*};

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct EntryValidationArgs {
    pub validation_data: EntryValidationData<Entry>,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct AgentIdValidationArgs {
    pub validation_data: EntryValidationData<AgentId>,
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
