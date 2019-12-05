use holochain_core_types::error::HolochainError;
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_locksmith::RwLock;
use holochain_wasm_utils::api_serialization::crypto::CryptoMethod;
use jsonrpc_core::IoHandler;
use jsonrpc_lite::JsonRpc;
use serde_json;
use snowflake::ProcessUniqueId;
use std::{fmt, sync::Arc};

#[derive(Clone)]
pub struct ConductorApi(Arc<RwLock<IoHandler>>);

#[derive(Clone, Serialize, Deserialize, Debug, DefaultJson)]
struct JsonRpcParams {
    #[serde(rename = "payload")]
    payload: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, DefaultJson)]
struct JsonRpcRequest {
    #[serde(rename = "jsonrpc")]
    jsonrpc: String,
    #[serde(rename = "method")]
    method: String,
    #[serde(rename = "params")]
    params: JsonRpcParams,
    #[serde(rename = "id")]
    id: String,
}

impl JsonRpcRequest {
    fn new(request: &str, payload: &str) -> Self {
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: format!("agent/{}", request),
            params: JsonRpcParams {
                payload: payload.to_owned(),
            },
            id: format!("{}", ProcessUniqueId::new()),
        }
    }
}

pub fn send_json_rpc(
    handle: Arc<RwLock<IoHandler>>,
    payload: String,
    request_response: (String, String),
) -> Result<String, HolochainError> {
    let handler = handle.write().unwrap();

    let (request, _) = request_response;
    let json_rpc_request = JsonRpcRequest::new(&request, &payload);

    let response = handler
        .handle_request_sync(&String::from(JsonString::from(json_rpc_request)))
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
        send_json_rpc(self.0.clone(), payload, request_response)
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
