use holochain_core_types::hash::HashString;
use holochain_core_types::json::*;

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
