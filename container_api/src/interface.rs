use holochain_core::state::State;
use holochain_core_types::{cas::content::Address, dna::capabilities::CapabilityCall};
use Holochain;

use jsonrpc_ws_server::jsonrpc_core::{self, types::params::Params, IoHandler, Value};
use serde_json;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{mpsc::Receiver, Arc, RwLock},
};

use config::{DnaConfiguration, InstanceConfiguration, StorageConfiguration};
use container::CONTAINER;
use container_admin::ContainerAdmin;
use serde_json::map::Map;

pub type InterfaceError = String;
pub type InstanceMap = HashMap<String, Arc<RwLock<Holochain>>>;

pub trait DispatchRpc {
    fn handler(self) -> IoHandler;
}

macro_rules! container_call {
    ( |$container:ident| $call_expr:expr ) => {
        match * CONTAINER.lock().unwrap() {
            Some( ref mut $container) => {
                $call_expr
                    .map_err( | e | {
                        let mut new = jsonrpc_core::Error::internal_error();
                        new.message = e.to_string();
                        new
                    })
            }
            None => {
                println!("Admin container function called without a container mounted as singleton!");
                // If interfaces are supposed to work, the container needs to be mounted to a static place
                // with container_api::container::mount_container_from_config(config: Configuration).
                // There are cases in which we don't want to treat the container as a singleton such as
                // holochain_nodejs and tests in particular. In those cases, calling admin functions via
                // interfaces (websockt/http) won't work, but also we don't need that.
                let mut error = jsonrpc_core::Error::internal_error();
                error.message = String::from(
                    "Admin container function called without a container mounted as singleton!",
                );
                Err(error)
            },
        }

    }
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

    fn unwrap_params_map(params: Params) -> Result<Map<String, Value>, jsonrpc_core::Error> {
        match params {
            Params::Map(map) => Ok(map),
            _ => Err(jsonrpc_core::Error::invalid_params("expected params map")),
        }
    }

    fn get_as_string<T: Into<String>>(
        key: T,
        params_map: &Map<String, Value>,
    ) -> Result<String, jsonrpc_core::Error> {
        let key = key.into();
        Ok(params_map
            .get(&key)
            .ok_or(jsonrpc_core::Error::invalid_params(format!(
                "`{}` param not provided",
                &key
            )))?
            .as_str()
            .ok_or(jsonrpc_core::Error::invalid_params(format!(
                "`{}` is not a valid json string",
                &key
            )))?
            .to_string())
    }

    pub fn with_admin_dna_functions(mut self) -> Self {
        self.io
            .add_method("admin/dna/install_from_file", move |params| {
                let params_map = Self::unwrap_params_map(params)?;

                let id = Self::get_as_string("id", &params_map)?;
                let path = Self::get_as_string("path", &params_map)?;

                container_call!(|c| c.install_dna_from_file(PathBuf::from(path), id.to_string()))?;

                Ok(serde_json::Value::String("success".into()))
            });

        self.io.add_method("admin/dna/uninstall", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.uninstall_dna(&id))?;
            Ok(serde_json::Value::String("success".into()))
        });

        self.io.add_method("admin/dna/list", move |_params| {
            let dnas = container_call!(|c| Ok(c.config.dnas.clone()) as Result<Vec<DnaConfiguration>, String>)?;
            Ok(serde_json::Value::Array(dnas.iter()
                .map(|dna| json!({"id": dna.id, "hash": dna.hash}))
                .collect()
            ))
        });

        self.io.add_method("admin/instance/add", move |params| {
            let params_map = Self::unwrap_params_map(params)?;

            let id = Self::get_as_string("id", &params_map)?;
            let dna_id = Self::get_as_string("dna_id", &params_map)?;
            let agent_id = Self::get_as_string("agent_id", &params_map)?;

            let new_instance = InstanceConfiguration {
                id: id.to_string(),
                dna: dna_id.to_string(),
                agent: agent_id.to_string(),
                storage: StorageConfiguration::Memory, // TODO: don't actually use this. Have some idea of default store
            };

            container_call!(|c| c.add_instance(new_instance))?;

            Ok(serde_json::Value::String("success".into()))
        });

        self.io.add_method("admin/instance/remove", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.remove_instance(&id))?;
            Ok(serde_json::Value::String("success".into()))
        });

        self.io.add_method("admin/instance/start", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.start_instance(&id))?;
            Ok(serde_json::Value::String("success".into()))
        });

        self.io.add_method("admin/instance/stop", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.stop_instance(&id))?;
            Ok(serde_json::Value::String("success".into()))
        });

        self.io.add_method("admin/instance/list", move |_params| {
            let instances = container_call!(|c| Ok(c.config.instances.clone()) as Result<Vec<InstanceConfiguration>, String>)?;
            Ok(serde_json::Value::Array(instances.iter()
                .map(|instance|
                    json!({
                        "id": instance.id,
                        "dna": instance.dna,
                        "agent": instance.agent,
                    }))
                .collect()
            ))
        });

        self
    }
}

pub trait Interface {
    fn run(&self, handler: IoHandler, kill_switch: Receiver<()>) -> Result<(), String>;
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
