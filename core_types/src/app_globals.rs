use hash::HashString;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppGlobals {
    pub app_name: String,
    pub app_dna_hash: HashString,
    pub app_agent_id_str: String,
    pub app_agent_key_hash: HashString,
    pub app_agent_initial_hash: HashString,
    pub app_agent_latest_hash: HashString,
}