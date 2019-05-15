use holochain_core_types::error::HolochainError;
use jsonrpc_core::IoHandler;
use jsonrpc_lite::JsonRpc;
use snowflake::ProcessUniqueId;
use std::{
    fmt,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct ConductorApi(Arc<RwLock<IoHandler>>);

impl ConductorApi {
    pub fn new(conductor_api: Arc<RwLock<IoHandler>>) -> ConductorApi {
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
            JsonRpc::Success(_) => Ok(String::from(response.get_result()?["signature"].as_str()?)),
            JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(serde_json::to_string(
                &response.get_error()?,
            )?)),
            _ => Err(HolochainError::ErrorGeneric("Signing failed".to_string())),
        }
    }

    pub fn get(&self) -> &Arc<RwLock<IoHandler>> {
        &self.0
    }
}

impl PartialEq for ConductorApi {
    fn eq(&self, _other: &ConductorApi) -> bool {
        false
    }
}

impl fmt::Debug for ConductorApi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
