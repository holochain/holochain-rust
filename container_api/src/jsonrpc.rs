use std::convert::TryFrom;

use serde_json::{self, Value};

use error::{HolochainInstanceError, HolochainResult};
use holochain::Holochain;
use holochain_core::{context::Context, logger::SimpleLogger, persister::SimplePersister};
use holochain_core_types::{
    cas::content::AddressableContent, error::HolochainError, json::JsonString,
};

use config::InterfaceConfiguration;

#[derive(Serialize, Deserialize)]
pub struct JsonRpc {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u32,
}

impl TryFrom<String> for JsonRpc {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let msg: JsonRpc = serde_json::from_str(&s).map_err(|e| e.to_string())?;
        if msg.jsonrpc != "2.0" {
            Err("JSONRPC version must be 2.0".to_string())
        } else {
            Ok(msg)
        }
    }
}
