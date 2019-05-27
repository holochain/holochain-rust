use holochain_core_types::{error::HolochainError, json::*};

#[derive(Serialize, Deserialize, Debug, DefaultJson, Default)]
pub struct InitParams {
    pub params: JsonString,
}
