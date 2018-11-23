use holochain_core_types::json::JsonString;
use Holochain;

use jsonrpc::{jsonrpc_error, jsonrpc_success, JsonRpcRequest};
use serde_json;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use config::{Configuration, InstanceConfiguration};

pub type InterfaceError = String;
pub type InstanceMap = HashMap<String, Arc<Mutex<Holochain>>>;

pub trait DispatchRpc {
    fn dispatch_rpc(&self, rpc: &JsonRpcRequest) -> JsonString;
}

pub struct RpcDispatcher {
    instances: InstanceMap,
    instance_configs: HashMap<String, InstanceConfiguration>,
}

unsafe impl Send for RpcDispatcher {}

impl RpcDispatcher {
    pub fn new(config: &Configuration, instances: InstanceMap) -> Self {
        let instance_configs = config
            .instances
            .iter()
            .map(|inst| (inst.id.clone(), inst.clone()))
            .collect();
        Self {
            instances,
            instance_configs,
        }
    }
}

/// Implements routing for JSON-RPC calls:
/// {instance_id}/{zome}/{cap}/{func} -> a zome call
/// info/list_instances               -> Map of InstanceConfigs, keyed by ID
/// admin/...                         -> TODO
impl DispatchRpc for RpcDispatcher {
    /// Dispatch to the correct Holochain and `call` it based on the JSONRPC method
    fn dispatch_rpc(&self, rpc: &JsonRpcRequest) -> JsonString {
        let matches: Vec<&str> = rpc.method.trim_matches('/').split('/').collect();
        let result = match matches.as_slice() {
            // A normal zome function call
            [instance_id, zome, cap, func] => {
                let key = instance_id.to_string();
                self.instances
                    .get(&key)
                    .ok_or(format!("No instance with ID: {:?}", key))
                    .and_then(|hc_mutex| {
                        let mut hc = hc_mutex.lock().unwrap();
                        hc.call(zome, cap, func, &rpc.params.to_string())
                            .map_err(|e| e.to_string())
                    })
            }

            // get all instance config info
            ["info", "instances"] => serde_json::to_string(&self.instance_configs)
                .map(JsonString::from)
                .map_err(|e| e.to_string()),

            // unknown method
            _ => Err(format!("bad rpc method: {}", rpc.method)),
        };
        result
            .map(|r| jsonrpc_success(rpc.id, r))
            .unwrap_or_else(|e| jsonrpc_error(rpc.id, e))
    }
}

pub trait Interface {
    fn run(&self, Arc<DispatchRpc>) -> Result<(), String>;
}
