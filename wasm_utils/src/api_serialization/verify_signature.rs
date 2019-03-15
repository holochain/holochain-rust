use holochain_core_types::{error::HolochainError, json::*, signature::Provenance};

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct VerifySignatureArgs {
    pub provenance: Provenance,
    pub payload: String,
}
