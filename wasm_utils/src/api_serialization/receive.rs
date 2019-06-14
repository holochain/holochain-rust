use holochain_persistence_api::cas::content::Address;
use holochain_json_api::{error::JsonError, json::*};


#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct ReceiveParams {
    pub from: Address,
    pub payload: String,
}
