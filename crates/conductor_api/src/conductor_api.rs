use holochain_core_types::error::HolochainError;
use holochain_locksmith::RwLock;
use holochain_wasm_utils::api_serialization::crypto::CryptoMethod;
use jsonrpc_core::IoHandler;
use jsonrpc_lite::JsonRpc;
use serde_json;
use snowflake::ProcessUniqueId;
use std::{fmt, sync::Arc};

#[derive(Clone)]
pub struct ConductorApi(Arc<RwLock<IoHandler>>);

pub fn send_json_rpc(
    handle: Arc<RwLock<IoHandler>>,
    payload: String,
    request_response: (String, String),
) -> Result<String, HolochainError> {
    let handler = handle.write().unwrap();

    let (request, _) = request_response;

    let naive_request = format!(
        r#"{{"jsonrpc": "2.0", "method": "agent/{}", "params": {{"payload": "{}"}}, "id": "{}"}}"#,
        request,
        payload,
        ProcessUniqueId::new(),
    );

    let response = handler
        .handle_request_sync(&naive_request)
        .ok_or_else(|| format!("Conductor request agent/{} failed", request))?;

    let response = JsonRpc::parse(&response)?;

    match response {
        JsonRpc::Success(_) => Ok(String::from(
            response.get_result()?[request_response.1].as_str()?,
        )),
        JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(serde_json::to_string(
            &response.get_error()?,
        )?)),
        _ => Err(HolochainError::ErrorGeneric(format!(
            "agent/{} failed",
            request,
        ))),
    }
}

impl ConductorApi {
    pub fn new(conductor_api: Arc<RwLock<IoHandler>>) -> ConductorApi {
        ConductorApi(conductor_api)
    }

    pub fn execute(&self, payload: String, method: CryptoMethod) -> Result<String, HolochainError> {
        let request_response = match method {
            CryptoMethod::Sign => (String::from("sign"), String::from("signature")),
            CryptoMethod::Encrypt => (String::from("encrypt"), String::from("message")),
            CryptoMethod::Decrypt => (String::from("decrypt"), String::from("message")),
        };
        // all crypto payloads are base64 encoded as we need to support arbitrary data and JSON
        // handling is painful without some kind of encoding
        let encoded_payload = base64::encode(&payload);
        // let encoded_payload = payload;
        send_json_rpc(self.0.clone(), encoded_payload, request_response)
    }

    pub fn get(&self) -> &Arc<RwLock<IoHandler>> {
        &self.0
    }

    pub fn reset(&self, api: IoHandler) {
        *self.0.write().unwrap() = api;
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
