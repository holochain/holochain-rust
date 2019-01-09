use holochain_core::state::State;
use holochain_core_types::{cas::content::Address, dna::capabilities::CapabilityCall};
use Holochain;

use jsonrpc_ws_server::jsonrpc_core::{self, IoHandler, Value, types::params::Params};
use serde_json;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use config::InstanceConfiguration;

pub type InterfaceError = String;
pub type InstanceMap = HashMap<String, Arc<RwLock<Holochain>>>;

pub trait DispatchRpc {
    fn handler(self) -> IoHandler;
}

/// ContainerApiBuilder creates IoHandlers that implement RPCs for exposure
/// through interfaces or bridges.
/// This includes zome function calls as well as admin functionality.
///
/// Examples for method names are:
/// {instance_id}/{zome}/{cap}/{func} -> a zome call
/// info/list_instances               -> Map of InstanceConfigs, keyed by ID
/// admin/...                         -> TODO
///
/// Each interface has their own handler, and each may be configured differently.
/// This builder makes it convenient to create handlers with different configurations.
///
/// Call any sequence of with_* functions on a ContainerApiBuilder object and finalize
/// with spawn() to retrieve the IoHandler.
pub struct ContainerApiBuilder {
    instances: InstanceMap,
    instance_configs: HashMap<String, InstanceConfiguration>,
    io: Box<IoHandler>,
}

impl ContainerApiBuilder {
    pub fn new() -> Self {
        ContainerApiBuilder {
            instances: HashMap::new(),
            instance_configs: HashMap::new(),
            io: Box::new(IoHandler::new()),
        }
    }

    /// Finish the building and retrieve the populated handler
    pub fn spawn(mut self) -> IoHandler {
        self.setup_info_api();
        *self.io
    }

    /// Adds a "info/instances" method that returns a JSON object describing all registered
    /// instances we have a config for.
    fn setup_info_api(&mut self) {
        let instance_configs = self.instance_configs.clone();

        let configs: Vec<_> = self
            .instances
            .iter()
            .filter(|&(name, _)| instance_configs.contains_key(name))
            .map(|(name, _)| instance_configs.get(name).unwrap())
            .collect();

        let config_string = serde_json::to_string(&configs)
            .expect("Vector of InstanceConfigurations must be serializable");

        self.io.add_method("info/instances", move |_| {
            Ok(Value::String(config_string.clone()))
        });
    }

    /// Add a [InstanceConfig](struct.InstanceConfig.html) for a custom named instance
    pub fn with_named_instance_config(
        mut self,
        instance_name: String,
        instance_config: InstanceConfiguration,
    ) -> Self {
        self.instance_configs.insert(instance_name, instance_config);
        self
    }

    /// Add a vector of [InstanceConfig](struct.InstanceConfig.html) and regard their ID from
    /// the config as name.
    pub fn with_instance_configs(mut self, instance_configs: Vec<InstanceConfiguration>) -> Self {
        for config in instance_configs {
            self.instance_configs.insert(config.id.clone(), config);
        }
        self
    }

    /// Add several instances with the names given in the InstanceMap
    pub fn with_instances(mut self, instances: InstanceMap) -> Self {
        for (instance_id, hc_lock) in instances {
            self = self.with_named_instance(instance_id, hc_lock);
        }
        self
    }

    /// Add a single instance and register it under the given name
    pub fn with_named_instance(
        mut self,
        instance_name: String,
        instance: Arc<RwLock<Holochain>>,
    ) -> Self {
        let hc_lock = instance.clone();
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
                                instance_name, zome_name, cap_name, func_name
                            );
                            let hc_lock_inner = hc_lock.clone();
                            self.io.add_method(&method_name, move |params| {
                                let mut hc = hc_lock_inner.write().unwrap();
                                let params_string =
                                    serde_json::to_string(&params).map_err(|e| {
                                        jsonrpc_core::Error::invalid_params(e.to_string())
                                    })?;
                                let response = hc
                                    .call(
                                        &zome_name,
                                        Some(CapabilityCall::new(
                                            cap_name.clone(),
                                            Address::from("fake_token"),
                                            None,
                                        )),
                                        &func_name,
                                        &params_string,
                                    )
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
        self.instances
            .insert(instance_name.clone(), instance.clone());
        self
    }

    pub fn with_admin_dna_functions(mut self) -> Self {
        self.io.add_method("admin/dna/install", move |params| {
            let params_map = match params {
                Params::Map(map) => Ok(map),
                _ => Err(String::from("Expected parameters map")),
            }?;
            let id = params_map.get("id")?;
            let path = params_map.get("file_path")?;
            let params_string =
                serde_json::to_string(&params).map_err(|e| {
                    jsonrpc_core::Error::invalid_params(e.to_string())
                })?;


        });
        self
    }
}

pub trait Interface {
    fn run(&self, handler: IoHandler) -> Result<(), String>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{config::Configuration, container::tests::test_container};

    fn example_config_and_instances() -> (Configuration, InstanceMap) {
        let container = test_container();
        let holochain = container
            .instances()
            .get("test-instance-1")
            .unwrap()
            .clone();
        let mut instances = InstanceMap::new();
        instances.insert("test-instance-1".into(), holochain);
        (container.config(), instances)
    }

    #[test]
    fn test_new_dispatcher() {
        let (config, instances) = example_config_and_instances();
        let handler = ContainerApiBuilder::new()
            .with_instances(instances.clone())
            .with_instance_configs(config.instances)
            .spawn();
        let result = format!("{:?}", handler).to_string();
        println!("{}", result);
        assert!(result.contains("info/instances"));
        assert!(result.contains(r#""test-instance-1/greeter/public/hello""#));
        assert!(!result.contains(r#""test-instance-2//test/test""#));
    }

    #[test]
    fn test_named_instances() {
        let (config, instances) = example_config_and_instances();
        let handler = ContainerApiBuilder::new()
            .with_named_instance(
                String::from("happ-store"),
                instances.iter().nth(0).unwrap().1.clone(),
            )
            .with_named_instance_config(
                String::from("happ-store"),
                config.instances.iter().nth(0).unwrap().clone(),
            )
            .spawn();
        let result = format!("{:?}", handler).to_string();
        println!("{}", result);
        assert!(result.contains("info/instances"));
        assert!(result.contains(r#""happ-store/greeter/public/hello""#));
        assert!(!result.contains(r#""test-instance-1//test/test""#));
    }
}
