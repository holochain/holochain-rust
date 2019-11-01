use holochain_core_types::{agent::Base32, signature::Signature};
use holochain_json_api::{error::JsonError, json::*};

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
