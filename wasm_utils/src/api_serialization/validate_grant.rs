use holochain_core_types::{cas::content::Address, error::HolochainError, json::*};

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct ValidateGrantParams {
    pub capability_id: String,
    pub assignees: Vec<Address>,
}
