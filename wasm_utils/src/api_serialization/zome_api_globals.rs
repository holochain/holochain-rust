use holochain_core_types::hash::HashString;

#[derive(Deserialize, Serialize, Clone)]
pub struct ZomeApiGlobals {
    pub dna_name: String,
    pub dna_hash: HashString,
    pub agent_id_str: String,
    pub agent_address: HashString,
    pub agent_initial_hash: HashString,
    pub agent_latest_hash: HashString,
}
