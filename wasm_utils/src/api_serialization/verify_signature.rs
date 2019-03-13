use holochain_core_types::{agent::Base32, error::HolochainError, json::*};

#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct VerifySignatureArgs {
    pub pub_key: Base32,
    pub payload: String,
    pub signature: String,
}
