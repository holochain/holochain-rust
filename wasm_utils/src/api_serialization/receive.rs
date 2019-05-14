use holochain_core_types::{cas::content::Address, error::HolochainError, json::*};

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct ReceiveParams {
    pub from: Address,
    pub payload: String,
}
