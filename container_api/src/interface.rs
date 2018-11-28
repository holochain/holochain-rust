use holochain_core::state::State;
use Holochain;

use jsonrpc_ws_server::jsonrpc_core::{self, IoHandler, Value};
use serde_json;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use config::{Configuration, InstanceConfiguration};

pub type InterfaceError = String;
pub type InstanceMap = HashMap<String, Arc<RwLock<Holochain>>>;

pub trait DispatchRpc {
    fn handler(self) -> IoHandler;
}

/// ContainerApiDispatcher exposes some subset of the Container API,
/// including zome function calls as well as admin functionality.
/// Each interface has their own dispatcher, and each may be configured differently.
pub struct ContainerApiDispatcher {
    instances: InstanceMap,
    pub io: IoHandler,
}

unsafe impl Send for ContainerApiDispatcher {}

/// Implements routing for JSON-RPC calls:
/// {instance_id}/{zome}/{cap}/{func} -> a zome call
/// info/list_instances               -> Map of InstanceConfigs, keyed by ID
/// admin/...                         -> TODO
impl ContainerApiDispatcher {
    pub fn new(config: &Configuration, instances: InstanceMap) -> Self {
        let instance_configs = config
            .instances
            .iter()
            .map(|inst| (inst.id.clone(), inst.clone()))
            .collect();
        let io = IoHandler::new();
        let mut this = Self { instances, io };
        this.setup_info_api(instance_configs);
        this.setup_zome_api();
        this
    }

    fn setup_info_api(&mut self, instance_configs: HashMap<String, InstanceConfiguration>) {
        self.io.add_method("info/instances", move |_| {
            let configs = instance_configs.clone();
            let config_string = serde_json::to_string(&configs)
                .map_err(|e| jsonrpc_core::Error::invalid_params(e.to_string()))?;
            Ok(Value::String(config_string))
        });
    }

    fn setup_zome_api(&mut self) {
        for (instance_id, hc_lock) in self.instances.clone() {
            let hc_lock = hc_lock.clone();
            let hc = hc_lock.read().unwrap();
            let state: State = hc.state().unwrap();
            let nucleus = state.nucleus();
            let dna = nucleus.dna();
            match dna {
                Some(dna) => {
                    for (zome_name, zome) in dna.zomes {
                        for (cap_name, cap) in zome.capabilities {
                            for func in cap.functions {
                                let func_name = func.name;
                                let zome_name = zome_name.clone();
                                let cap_name = cap_name.clone();
                                let method_name = format!(
                                    "{}/{}/{}/{}",
                                    instance_id, zome_name, cap_name, func_name
                                );
                                let hc_lock_inner = hc_lock.clone();
                                self.io.add_method(&method_name, move |params| {
                                    let mut hc = hc_lock_inner.write().unwrap();
                                    let params_string =
                                        serde_json::to_string(&params).map_err(|e| {
                                            jsonrpc_core::Error::invalid_params(e.to_string())
                                        })?;
                                    let response = hc
                                        .call(&zome_name, &cap_name, &func_name, &params_string)
                                        .map_err(|e| {
                                            jsonrpc_core::Error::invalid_params(e.to_string())
                                        })?;
                                    Ok(Value::String(response.to_string()))
                                })
                            }
                        }
                    }
                }
                None => unreachable!(),
            };
        }
    }
}

impl DispatchRpc for ContainerApiDispatcher {
    fn handler(self) -> IoHandler {
        self.io
    }
}

pub trait Interface<D: DispatchRpc> {
    fn run(&self, d: D) -> Result<(), String>;
}
