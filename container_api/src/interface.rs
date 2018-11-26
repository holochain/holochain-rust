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
use jsonrpc_ws_server::{
    jsonrpc_core::{self, IoHandler, MetaIoHandler, Metadata, Value}
};

use config::{Configuration, InstanceConfiguration};

pub type InterfaceError = String;
pub type InstanceMap = HashMap<String, Arc<RwLock<Holochain>>>;

pub trait DispatchRpc {
    fn handler(self) -> IoHandler;
}
// pub trait DispatchRpc<M: Metadata + Default> {
//     fn handler(&self) -> MetaIoHandler<M>;
// }

/// ContainerApiDispatcher exposes some subset of the Container API,
/// including zome function calls as well as admin functionality.
/// Each interface has their own dispatcher, and each may be configured differently.
pub struct ContainerApiDispatcher {
    instances: InstanceMap,
    instance_configs: HashMap<String, InstanceConfiguration>,
    io: IoHandler,
}

unsafe impl Send for ContainerApiDispatcher {}

impl ContainerApiDispatcher {
    pub fn new(config: &Configuration, instances: InstanceMap) -> Self {
        let instance_configs = config
            .instances
            .iter()
            .map(|inst| (inst.id.clone(), inst.clone()))
            .collect();
        let io = IoHandler::new();
        let mut this = Self {
            instances,
            instance_configs,
            io,
        };
        this.setup_api();
        this
    }

    fn setup_api(&mut self) {
        self.setup_info_api();
        self.setup_zome_api();
    }

    fn setup_info_api(&mut self) {
        self.io.add_method("info/instances", |_| {
            Ok(Value::String("TODO: instances".to_string()))
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
                    dna.zomes.into_iter().for_each(|(zome_name, zome)| {
                        zome.capabilities.into_iter().for_each(|(cap_name, cap)| {
                            cap.functions.into_iter().for_each(|func| {
                                let func_name = func.name;
                                let zome_name = zome_name.clone();
                                let cap_name = cap_name.clone();
                                let method_name = format!(
                                    "{}/{}/{}/{}",
                                    instance_id,
                                    zome_name,
                                    cap_name,
                                    func_name
                                );
                                let hc_lock_inner = hc_lock.clone();
                                self.io.add_method(&method_name, move |_params| {
                                    let mut hc = hc_lock_inner.write().unwrap();
                                    let response = hc.call(&zome_name, &cap_name, &func_name, "")
                                        .map_err(|e| jsonrpc_core::Error::invalid_params(e.to_string()))?;
                                    Ok(Value::String(response.to_string()))
                                });
                            })
                        })
                    });
                },
                None => unreachable!()
            }
        }
    }
}

/// Implements routing for JSON-RPC calls:
/// {instance_id}/{zome}/{cap}/{func} -> a zome call
/// info/list_instances               -> Map of InstanceConfigs, keyed by ID
/// admin/...                         -> TODO

impl DispatchRpc for ContainerApiDispatcher {
    fn handler(self) -> IoHandler { 
        self.io
    }
}

pub trait Interface<D: DispatchRpc> {
    fn run(&self, D) -> Result<(), String>;
}

// 
// impl DispatchRpc<()> for ContainerApiDispatcher {
// 
//     fn handler(&self) -> IoHandler<()> { 
//         self.io.into()
//     }
// 
// }
// 
// pub trait Interface<M: Metadata + Default> {
//     fn run(&self, Arc<DispatchRpc<M>>) -> Result<(), String>;
// }
