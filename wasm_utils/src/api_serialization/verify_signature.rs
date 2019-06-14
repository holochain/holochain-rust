use holochain_core_types::signature::Provenance;
use holochain_json_api::{error::JsonError, json::*};

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct VerifySignatureArgs {
    pub provenance: Provenance,
    pub payload: String,
}
