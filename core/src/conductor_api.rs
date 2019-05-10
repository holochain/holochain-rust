use std::sync::{Arc, RwLock};
use jsonrpc_core::IoHandler;
use holochain_core_types::error::HolochainError;
use snowflake::ProcessUniqueId;
use jsonrpc_lite::JsonRpc;

#[derive(Clone)]
pub struct ConductorApi(Arc<RwLock<IoHandler>>);

impl ConductorApi {
    pub fn new(conductor_api: Arc<RwLock<IoHandler>>) -> ConductorApi{
        ConductorApi(conductor_api)
    }

    pub fn sign(&self, payload: String) -> Result<String, HolochainError> {
        let handler = self.0.write().unwrap();
        let request = format!(
            r#"{{"jsonrpc": "2.0", "method": "agent/sign", "params": {{"payload": "{}"}}, "id": "{}"}}"#,
            payload, ProcessUniqueId::new(),
        );

        let response = handler
            .handle_request_sync(&request)
            .ok_or("Conductor sign call failed".to_string())?;

        let response = JsonRpc::parse(&response)?;

        match response {
            JsonRpc::Success(_) => Ok(String::from(
                response.get_result().unwrap()["signature"]
                    .as_str()
                    .unwrap(),
            )),
            JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(
                serde_json::to_string(&response.get_error().unwrap()).unwrap(),
            )),
            _ => Err(HolochainError::ErrorGeneric("Signing failed".to_string())),
        }
    }
}