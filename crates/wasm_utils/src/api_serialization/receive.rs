use holochain_json_api::{error::JsonError, json::*};
use holochain_persistence_api::cas::content::Address;

#[derive(Clone, Serialize, Deserialize, Debug, DefaultJson)]
pub struct ReceiveParams {
    pub from: Address,
    pub payload: String,
}
