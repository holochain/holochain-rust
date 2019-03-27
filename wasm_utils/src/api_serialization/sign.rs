use holochain_core_types::{agent::Base32, error::HolochainError, json::*, signature::Signature};

#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct SignArgs {
    pub payload: String,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct SignOneTimeResult {
    pub pub_key: Base32,
    pub signature: Signature,
}
