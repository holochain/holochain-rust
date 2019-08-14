use holochain_json_api::{error::JsonError, json::*};

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub enum MetaMethod {
    Version,
    Hash,
}
#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct MetaArgs {
    pub method: MetaMethod,
}
#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub enum MetaResult {
    Version(String),
    Hash(String),
}
