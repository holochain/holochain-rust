use holochain_core_types::dna::capabilities::CapabilityRequest;
use holochain_persistence_api::{hash::HashString, cas::content::Address};
use holochain_json_api::{error::JsonError, json::*};

#[derive(Deserialize, Serialize, Clone, Debug, DefaultJson)]
pub struct ZomeApiGlobals {
    pub dna_name: String,
    pub dna_address: Address,
    pub agent_id_str: String,
    pub agent_address: Address,
    pub agent_initial_hash: HashString,
    pub agent_latest_hash: HashString,
    pub public_token: Address,
    pub cap_request: Option<CapabilityRequest>,
    pub properties: JsonString,
}
