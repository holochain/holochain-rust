use holochain_core_types::{cas::content::Address, error::HolochainError, json::*, time::Timeout};

/// Struct for input data received when Zome API function send() is invoked
#[derive(Deserialize, Clone, PartialEq, Debug, Serialize, DefaultJson)]
pub struct SendArgs {
    pub to_agent: Address,
    pub payload: String,
    pub options: SendOptions,
}

#[derive(Deserialize, Clone, PartialEq, Debug, Serialize, DefaultJson)]
pub struct SendOptions(pub Timeout);
