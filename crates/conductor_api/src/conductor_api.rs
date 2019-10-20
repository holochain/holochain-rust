use holochain_core_types::{
    error::HolochainError,
    sync::{HcMutex as Mutex, HcRwLock as RwLock},
};
use holochain_wasm_utils::api_serialization::crypto::CryptoMethod;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_lite::JsonRpc;
use jsonrpc_pubsub::{PubSubHandler, Session, Sink, SubscriptionId};
use snowflake::ProcessUniqueId;
use std::{collections::HashMap, fmt, sync::Arc};

pub struct RpcHandler {
    pub io: PubSubHandler<Option<Arc<Session>>>,
    pub subscriptions: Mutex<HashMap<SubscriptionId, Sink>>,
}

impl RpcHandler {
    pub fn new() -> Self {
        Self {
            io: PubSubHandler::new(MetaIoHandler::default()),
            subscriptions: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Clone)]
pub struct ConductorApi(Arc<RwLock<RpcHandler>>);

pub fn send_json_rpc(
    handle: Arc<RwLock<RpcHandler>>,
    payload: String,
    request_reponse: (String, String),
) -> Result<String, HolochainError> {
    let handler = handle.write().unwrap();
    let request = format!(
        r#"{{"jsonrpc": "2.0", "method": "agent/{}", "params": {{"payload": "{}"}}, "id": "{}"}}"#,
        request_reponse.0,
        payload,
        ProcessUniqueId::new(),
    );

    let response = handler
        .io
        .handle_request_sync(&request, None)
        .ok_or_else(|| format!("Conductor request agent/{} failed", request_reponse.0))?;

    let response = JsonRpc::parse(&response)?;

    match response {
        JsonRpc::Success(_) => Ok(String::from(
            response.get_result()?[request_reponse.1].as_str()?,
        )),
        JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(serde_json::to_string(
            &response.get_error()?,
        )?)),
        _ => Err(HolochainError::ErrorGeneric(format!(
            "agent/{} failed",
            request_reponse.0
        ))),
    }
}

impl ConductorApi {
    pub fn new(conductor_api: Arc<RwLock<RpcHandler>>) -> ConductorApi {
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

    pub fn get(&self) -> &Arc<RwLock<RpcHandler>> {
        &self.0
    }

    pub fn reset(&self, api: RpcHandler) {
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
        write!(f, "<ConductorApi (opaque)>")
    }
}
