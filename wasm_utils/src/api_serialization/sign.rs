use holochain_core_types::{error::HolochainError, json::*};
use holochain_core_types::agent::Base32;
use holochain_core_types::signature::Signature;

#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct SignArgs {
    pub payload: String,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct SignOneTimeResult {
    pub pub_key: Base32,
    pub signature: Signature,
}
