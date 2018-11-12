use holochain_core_types::{error::HolochainError, hash::HashString, json::*};

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson)]
pub struct ZomeApiGlobals {
    pub dna_name: String,
    pub dna_hash: HashString,
    pub agent_id_str: String,
    pub agent_address: HashString,
    pub agent_initial_hash: HashString,
    pub agent_latest_hash: HashString,
}
