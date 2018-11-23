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
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u32,
}

pub fn jsonrpc_success(id: u32, result: JsonString) -> JsonString {
    json!({
        "jsonrpc": "2.0",
        // Question, is there a better way to safely go from JsonString into Value?
        "result": serde_json::from_str::<Value>(&result.to_string()).unwrap(),
        "id": id,
    }).into()
}

pub fn jsonrpc_error(id: u32, error: String) -> JsonString {
    json!({
        "jsonrpc": "2.0",
        "error": error,
        "id": id,
    }).into()
}

impl TryFrom<String> for JsonRpcRequest {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let msg: JsonRpcRequest = serde_json::from_str(&s).map_err(|e| e.to_string())?;
        if msg.jsonrpc != "2.0" {
            Err("JSONRPC version must be 2.0".to_string())
        } else {
            Ok(msg)
        }
    }
}
