use holochain_core_types::json::JsonString;
use holochain_core::state::State;
use Holochain;

use jsonrpc::{jsonrpc_error, jsonrpc_success, JsonRpcRequest};
use serde_json;
use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{Arc, RwLock, RwLockReadGuard},
    thread,
};
use jsonrpc_core;

use config::{Configuration, InstanceConfiguration};

pub type InterfaceError = String;
pub type InstanceMap = HashMap<String, Arc<RwLock<Holochain>>>;

pub trait DispatchRpc {
    fn dispatch_rpc(&self, rpc_string: &str) -> String;
}

/// ContainerApiDispatcher exposes some subset of the Container API,
/// including zome function calls as well as admin functionality.
/// Each interface has their own dispatcher, and each may be configured differently.
pub struct ContainerApiDispatcher {
    instances: InstanceMap,
    instance_configs: HashMap<String, InstanceConfiguration>,
    io: Box<jsonrpc_core::IoHandler>,
}

unsafe impl Send for ContainerApiDispatcher {}

impl ContainerApiDispatcher {
    pub fn new(config: &Configuration, instances: InstanceMap) -> Self {
        let instance_configs = config
            .instances
            .iter()
            .map(|inst| (inst.id.clone(), inst.clone()))
            .collect();
        let io = Box::new(jsonrpc_core::IoHandler::new());
        let self = Self {
            instances,
            instance_configs,
            io,
        };
        self.setup_api();
        self
    }

    fn setup_api(&mut self) {
        self.setup_info_api();
        self.setup_zome_api();
    }

    fn setup_info_api(&mut self) {
        self.io.add_method("info/instances", |_| {
            Ok(jsonrpc_core::Value::String(("TODO: instances".to_string())))
        });
    }

    fn setup_zome_api(&self) {
        for (instance_id, hc_lock) in self.instances.clone() {
            let mut hc = hc_lock.write().unwrap();
            let state: State = hc.state().unwrap();
            let nucleus = state.nucleus();
            nucleus.clone().dna.iter().map(|dna| {
                dna.zomes.iter().for_each(|(zome_name, zome)| {
                    zome.capabilities.iter().for_each(|(cap_name, cap)| {
                        cap.functions.iter().for_each(|func| {
                            let method_name = format!(
                                "{}/{}/{}/{}",
                                instance_id,
                                zome_name,
                                cap_name,
                                func.name
                            );
                            self.io.add_method(&method_name, |params: jsonrpc_core::Params| {
                                Ok(jsonrpc_core::Value::String(("hey".to_string())))
                            });
                        })
                    })
                });
            });
        }
    }
}

/// Implements routing for JSON-RPC calls:
/// {instance_id}/{zome}/{cap}/{func} -> a zome call
/// info/list_instances               -> Map of InstanceConfigs, keyed by ID
/// admin/...                         -> TODO
impl DispatchRpc for ContainerApiDispatcher {

    /// Dispatch to the correct Holochain and `call` it based on the JSONRPC method
    fn dispatch_rpc(&self, rpc_string: &str) -> String {
        self.io.handle_request_sync(rpc_string).ok_or(jsonrpc_core::Value::String("TODO".into())).and_then(|response: String| {
            // JsonString::try_from(response)
            Ok(response)
        }).unwrap_or_else(|e| "error (TODO)".into())
    }
}

pub trait Interface {
    fn run(&self, Arc<DispatchRpc>) -> Result<(), String>;
}
