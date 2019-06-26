use holochain_json_api::{error::JsonError, json::*};
#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct CryptoArgs {
    pub payload: String,
    pub method: ConductorCryptoApiMethod,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub enum ConductorCryptoApiMethod {
    Sign,
    Encrypt,
    Decrypt,
}
