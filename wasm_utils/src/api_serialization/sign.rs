use holochain_core_types::{agent::Base32, signature::Signature};

use lib3h_persistence_api::{error::PersistenceError, json::*};

#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct SignArgs {
    pub payload: String,
}

#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct OneTimeSignArgs {
    pub payloads: Vec<String>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct SignOneTimeResult {
    pub pub_key: Base32,
    pub signatures: Vec<Signature>,
}
