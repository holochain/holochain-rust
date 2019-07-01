use holochain_core_types::time::Timeout;
use holochain_json_api::{error::JsonError, json::*};
use holochain_persistence_api::cas::content::Address;

/// Struct for input data received when Zome API function send() is invoked
#[derive(Deserialize, Clone, PartialEq, Debug, Serialize, DefaultJson)]
pub struct SendArgs {
    pub to_agent: Address,
    pub payload: String,
    pub options: SendOptions,
}

#[derive(Deserialize, Clone, PartialEq, Debug, Serialize, DefaultJson)]
pub struct SendOptions(pub Timeout);
