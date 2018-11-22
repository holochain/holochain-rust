use holochain_core_types::{error::HolochainError, json::JsonString};
use Holochain;

use jsonrpc::JsonRpc;
use serde_json::{self, Value};
use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{Arc, Mutex},
    thread,
};
use ws::{self, Message, Result as WsResult};

pub type InterfaceError = String;
pub type InstanceMap = HashMap<String, Arc<Mutex<Holochain>>>;

pub trait DispatchRpc {
    fn dispatch_rpc(&self, rpc: &JsonRpc) -> Result<JsonString, HolochainError>;
}

pub struct RpcDispatcher {
    instances: InstanceMap,
}

unsafe impl Send for RpcDispatcher {}

impl RpcDispatcher {
    pub fn new(instances: InstanceMap) -> Self {
        Self { instances }
    }
}

impl DispatchRpc for RpcDispatcher {
    /// Dispatch to the correct Holochain and `call` it based on the JSONRPC method
    fn dispatch_rpc(&self, rpc: &JsonRpc) -> Result<JsonString, HolochainError> {
        let matches: Vec<&str> = rpc.method.split('/').collect();
        let result = if let [instance_id, zome, cap, func] = matches.as_slice() {
            let key = instance_id.to_string();
            self.instances
                .get(&key)
                .ok_or(format!("No instance with ID: {:?}", key))
                .and_then(|hc_mutex| {
                    let mut hc = hc_mutex.lock().unwrap();
                    hc.call(zome, cap, func, &rpc.params.to_string())
                        .map_err(|e| e.to_string())
                })
        } else {
            Err(format!("bad rpc method: {}", rpc.method))
        };
        result.map_err(HolochainError::ErrorGeneric)
    }
}

pub trait Interface {
    fn run(&self, Arc<DispatchRpc>) -> Result<(), String>;
}
