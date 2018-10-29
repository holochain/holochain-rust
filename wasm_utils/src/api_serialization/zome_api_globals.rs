use holochain_core_types::hash::HashString;
use holochain_core_types::json::*;
use holochain_core_types::error::HolochainError;
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ZomeApiGlobals {
    pub dna_name: String,
    pub dna_hash: HashString,
    pub agent_id_str: String,
    pub agent_address: HashString,
    pub agent_initial_hash: HashString,
    pub agent_latest_hash: HashString,
}

impl From<ZomeApiGlobals> for JsonString {
    fn from(v: ZomeApiGlobals) -> Self {
        default_to_json(v)
    }
}

impl TryFrom<JsonString> for ZomeApiGlobals {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}
