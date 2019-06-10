use holochain_core_types::signature::Provenance;
use lib3h_persistence_api::{error::PersistenceError, json::*};

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct VerifySignatureArgs {
    pub provenance: Provenance,
    pub payload: String,
}
