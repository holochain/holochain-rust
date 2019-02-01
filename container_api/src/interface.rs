use holochain_core::{nucleus::ribosome::fn_call::make_cap_call, state::State};
use holochain_core_types::cas::content::Address;
use Holochain;

use jsonrpc_ws_server::jsonrpc_core::{self, types::params::Params, IoHandler, Value};
use serde_json;
use std::{
    collections::HashMap,
    convert::TryFrom,
    path::PathBuf,
    sync::{mpsc::Receiver, Arc, RwLock},
};

use config::{
    AgentConfiguration, Bridge, DnaConfiguration, InstanceConfiguration, InterfaceConfiguration,
    InterfaceDriver,
};
use container::{ContainerAdmin, CONTAINER};
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
/// admin/...                         -> see [with_admin_dna_functions]
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

        let configs: Vec<serde_json::Value> = self
            .instances
            .iter()
            .filter(|&(name, _)| instance_configs.contains_key(name))
            .map(|(name, _)| instance_configs.get(name).unwrap())
            .map(|instance| {
                json!({
                    "id": instance.id,
                    "dna": instance.dna,
                    "agent": instance.agent,
                })
            })
            .collect();

        self.io.add_method("info/instances", move |_| {
            Ok(serde_json::Value::Array(configs.clone()))
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
                    for fn_decl in zome.fn_declarations {
                        let func_name = String::from(fn_decl.name);
                        let zome_name = zome_name.clone();
                        let method_name = format!("{}/{}/{}", instance_name, zome_name, func_name);
                        let hc_lock_inner = hc_lock.clone();
                        self.io.add_method(&method_name, move |params| {
                            let mut hc = hc_lock_inner.write().unwrap();
                            let params_string = serde_json::to_string(&params)
                                .map_err(|e| jsonrpc_core::Error::invalid_params(e.to_string()))?;

                            let cap_call = {
                                // TODO: get the token from the paramters on only if not there
                                // assume public token.  Also cleanup unwraps.
                                let context = hc.context();
                                let state = context.state().unwrap().nucleus();
                                let init = state.initialization().unwrap();
                                let token = init.get_public_token(&zome_name).unwrap();
                                let caller = Address::from("fake");
                                make_cap_call(
                                    context.clone(),
                                    token,
                                    caller,
                                    &func_name,
                                    params_string.clone(),
                                )
                            };

                            // TODO: need to get the caller identity in here somehow

                            let response = hc
                                .call(&zome_name, cap_call, &func_name, &params_string)
                                .map_err(|e| jsonrpc_core::Error::invalid_params(e.to_string()))?;
                            Ok(Value::String(response.to_string()))
                        })
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

    fn get_as_bool<T: Into<String>>(
        key: T,
        params_map: &Map<String, Value>,
    ) -> Result<bool, jsonrpc_core::Error> {
        let key = key.into();
        Ok(params_map
            .get(&key)
            .ok_or(jsonrpc_core::Error::invalid_params(format!(
                "`{}` param not provided",
                &key
            )))?
            .as_bool()
            .ok_or(jsonrpc_core::Error::invalid_params(format!(
                "`{}` has to be a boolean",
                &key
            )))?)
    }

    fn get_as_int<T: Into<String>>(
        key: T,
        params_map: &Map<String, Value>,
    ) -> Result<i64, jsonrpc_core::Error> {
        let key = key.into();
        Ok(params_map
            .get(&key)
            .ok_or(jsonrpc_core::Error::invalid_params(format!(
                "`{}` param not provided",
                &key
            )))?
            .as_i64()
            .ok_or(jsonrpc_core::Error::invalid_params(format!(
                "`{}` has to be an integer",
                &key
            )))?)
    }

    /// This adds functions to remotely change any aspect of the container config.
    /// After any change the container's config file gets saved.
    /// It is guaranteed that the config is either valid after the change or the change
    /// does not get applied but instead an error is reported back.
    ///
    ///  Full list of functions:
    ///
    ///  * `admin/dna/install_from_file`:
    ///     Installs a DNA from a given local file.
    ///     Params:
    ///     * `id`: [string] internal handle/name of the newly created DNA config
    ///     * `path`: [string] local file path to DNA file
    ///
    ///  * `admin/dna/uninstall`
    ///     Uninstalls a DNA from the container config. Recursively also removes (and stops)
    ///     all instances this DNA is used in.
    ///     Params:
    ///     * `id`: [string] handle of the DNA to be deleted.
    ///
    ///  * `admin/dna/list`
    ///     Returns an array of all configured DNAs.
    ///
    ///  * `admin/instance/add`
    ///     Creates a new instance and adds it to the config.
    ///     Does not start the instance nor add it to an interface
    ///     (see `admin/instance/start` and `admin/interface/add_instance`).
    ///     Params:
    ///     * `id`: [string] Name for the new instance
    ///     * `agent_id`: [string] Agent to run this instance with
    ///     * `dna_id`: [string] DNA to run in this instance
    ///
    ///  * `admin/instance/remove`
    ///     Removes an instance. Also remove its any uses of it in interfaces.
    ///     * `id`: [string] Which instance to remove?
    ///
    ///  * `admin/instance/start`
    ///     Starts a stopped instance or reports an error if the given instance is
    ///     running already
    ///     Params:
    ///     * `id`: [string] Which instance to start?
    ///
    ///  * `admin/instance/stop`
    ///     Stops a running instance or reports an error if the given instance is not running.
    ///     Params:
    ///     * `id`: [string] Which instance to stop?
    ///
    ///  * `admin/instance/list`
    ///     Returns an array of all instances that are configured.
    ///
    ///  * `admin/instance/running`
    ///     Returns an array of all instances that are running.
    ///
    ///  * `admin/interface/add`
    ///     Adds a new DNA / zome / container interface (that provides access to zome functions
    ///     of selected instances and container functions, depending on the interfaces config).
    ///     This also automatically starts the interface. Different from instances, there are no
    ///     *stopped* interfaces - every interface that is configured is also active.
    ///     Params:
    ///     * `id`: [string] ID for the new interface
    ///     * `admin`: [bool] Grant access to (these) admin functions?
    ///     * `type`: [string] Either "websocket" or "http"
    ///     * `port`:  [number] Port to bind the server to.
    ///
    ///  * `admin/interface/remove`
    ///     Remove an interface from config. This automatically stops the interface as well.
    ///     Params:
    ///     * `id`: [string] Which interface to stop?
    ///
    ///  * `admin/interface/add_instance`
    ///     Make a given DNA instance available via a given interface.
    ///     This restarts the given interface in order to have the change take effect.
    ///     Params:
    ///     * `interface_id`: Which interface to add the instance to?
    ///     * `instance_id`: Which instance to add?
    ///
    ///  * `admin/interface/remove_instance`
    ///     Remove an instance from a given interface.
    ///     This restarts the given interface in order to have the change take effect.
    ///     Params:
    ///     * `interface_id`: Which interface to remove the instance from?
    ///     * `instance_id`: Which instance to remove?
    ///
    ///  * `admin/interface/list`
    ///     Returns an array of all DNA/zome interfaces.
    ///
    ///  * `admin/agent/add`
    ///     Add an agent to the container configuration that can be used with instances.
    ///     Params:
    ///     * `id`: Handle of this agent configuration as used in the config / other function calls
    ///     * `name`: Nickname of this agent configuration
    ///     * `public_address`: Public part of this agents key. Has to match the private key in the
    ///         given key file.
    ///     * `key_file`: Local path to the file that holds this agent configuration's private key
    ///
    ///  * `admin/agent/remove`
    ///     Remove an agent from the container config.
    ///     Params:
    ///     * `id`: Which agent to remove?
    ///
    ///  * `admin/agent/list`
    ///     Returns an array of all configured agents.
    ///
    ///  * `admin/bridge/add`
    ///     Add a bridge between two instances to enable the caller to call the callee's
    ///     zome functions.
    ///     Params:
    ///     * `caller_id`: ID of the instance that will be able to call into the other instance
    ///     * `callee_id`: ID of the instance which's zome functions can be called
    ///     * `handle`: Name that the caller uses to reference this bridge and therefore the other
    ///             instance.
    ///
    ///  * `admin/bridge/remove`
    ///     Remove a bridge
    ///     Params:
    ///     * `caller_id`: ID of the instance that can call into the other instance
    ///     * `callee_id`: ID of the instance which's zome functions can be called
    ///
    ///  * `admin/bridge/list`
    ///     Returns an array of all bridges.
    ///
    pub fn with_admin_dna_functions(mut self) -> Self {
        self.io
            .add_method("admin/dna/install_from_file", move |params| {
                let params_map = Self::unwrap_params_map(params)?;
                let id = Self::get_as_string("id", &params_map)?;
                let path = Self::get_as_string("path", &params_map)?;
                let copy = Self::get_as_bool("copy", &params_map).unwrap_or(false);
                let properties = params_map.get("properties");
                container_call!(|c| c.install_dna_from_file(
                    PathBuf::from(path),
                    id.to_string(),
                    copy,
                    properties
                ))?;
                Ok(json!({"success": true}))
            });

        self.io.add_method("admin/dna/uninstall", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.uninstall_dna(&id))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/dna/list", move |_params| {
            let dnas =
                container_call!(|c| Ok(c.config().dnas) as Result<Vec<DnaConfiguration>, String>)?;
            Ok(serde_json::Value::Array(
                dnas.iter()
                    .map(|dna| json!({"id": dna.id, "hash": dna.hash}))
                    .collect(),
            ))
        });

        self.io.add_method("admin/instance/add", move |params| {
            let params_map = Self::unwrap_params_map(params)?;

            let id = Self::get_as_string("id", &params_map)?;
            let dna_id = Self::get_as_string("dna_id", &params_map)?;
            let agent_id = Self::get_as_string("agent_id", &params_map)?;
            container_call!(|c| c.add_instance(&id, &dna_id, &agent_id))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/instance/remove", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.remove_instance(&id))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/instance/start", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.start_instance(&id))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/instance/stop", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.stop_instance(&id))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/instance/list", move |_params| {
            let instances = container_call!(
                |c| Ok(c.config().instances) as Result<Vec<InstanceConfiguration>, String>
            )?;
            Ok(serde_json::Value::Array(
                instances
                    .iter()
                    .map(|instance| {
                        json!({
                            "id": instance.id,
                            "dna": instance.dna,
                            "agent": instance.agent,
                        })
                    })
                    .collect(),
            ))
        });

        self.io
            .add_method("admin/instance/running", move |_params| {
                let active_ids = container_call!(|c| Ok(c
                    .instances()
                    .iter()
                    .filter(|(_, hc)| hc.read().unwrap().active())
                    .map(|(id, _)| id)
                    .cloned()
                    .collect())
                    as Result<Vec<String>, String>)?;
                let instances = container_call!(
                    |c| Ok(c.config().instances) as Result<Vec<InstanceConfiguration>, String>
                )?;
                Ok(serde_json::Value::Array(
                    instances
                        .iter()
                        .filter(|instance| active_ids.contains(&instance.id))
                        .map(|instance| {
                            json!({
                                "id": instance.id,
                                "dna": instance.dna,
                                "agent": instance.agent,
                            })
                        })
                        .collect(),
                ))
            });

        self.io.add_method("admin/interface/add", move |params| {
            let params_map = Self::unwrap_params_map(params)?;

            let id = Self::get_as_string("id", &params_map)?;
            let admin = Self::get_as_bool("admin", &params_map)?;
            let driver_type = Self::get_as_string("type", &params_map)?;
            let port = u16::try_from(Self::get_as_int("port", &params_map)?).map_err(|_| {
                jsonrpc_core::Error::invalid_params(String::from(
                    "`port` has to be a 16bit integer",
                ))
            })?;

            let new_interface = InterfaceConfiguration {
                id: id.to_string(),
                admin,
                driver: match driver_type.as_ref() {
                    "websocket" => InterfaceDriver::Websocket { port },
                    "http" => InterfaceDriver::Http { port },
                    _ => {
                        return Err(jsonrpc_core::Error::invalid_params(String::from(
                            "`type` has to be either `websocket` or `http`",
                        )));
                    }
                },
                instances: Vec::new(),
            };

            container_call!(|c| c.add_interface(new_interface))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/interface/remove", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.remove_interface(&id))?;
            Ok(json!({"success": true}))
        });

        self.io
            .add_method("admin/interface/add_instance", move |params| {
                let params_map = Self::unwrap_params_map(params)?;
                let interface_id = Self::get_as_string("interface_id", &params_map)?;
                let instance_id = Self::get_as_string("instance_id", &params_map)?;
                container_call!(|c| c.add_instance_to_interface(&interface_id, &instance_id))?;
                Ok(json!({"success": true}))
            });

        self.io
            .add_method("admin/interface/remove_instance", move |params| {
                let params_map = Self::unwrap_params_map(params)?;
                let interface_id = Self::get_as_string("interface_id", &params_map)?;
                let instance_id = Self::get_as_string("instance_id", &params_map)?;
                container_call!(|c| c.remove_instance_from_interface(&interface_id, &instance_id))?;
                Ok(json!({"success": true}))
            });

        self.io.add_method("admin/interface/list", move |_params| {
            let interfaces = container_call!(
                |c| Ok(c.config().interfaces) as Result<Vec<InterfaceConfiguration>, String>
            )?;
            Ok(serde_json::to_value(interfaces)
                .map_err(|_| jsonrpc_core::Error::internal_error())?)
        });

        self.io.add_method("admin/agent/add", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            let name = Self::get_as_string("name", &params_map)?;
            let public_address = Self::get_as_string("public_address", &params_map)?;
            let key_file = Self::get_as_string("key_file", &params_map)?;

            let agent = AgentConfiguration {
                id,
                name,
                public_address,
                key_file,
            };
            container_call!(|c| c.add_agent(agent))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/agent/remove", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let id = Self::get_as_string("id", &params_map)?;
            container_call!(|c| c.remove_agent(&id))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/agent/list", move |_params| {
            let agents = container_call!(
                |c| Ok(c.config().agents) as Result<Vec<AgentConfiguration>, String>
            )?;
            Ok(serde_json::to_value(agents).map_err(|_| jsonrpc_core::Error::internal_error())?)
        });

        self.io.add_method("admin/bridge/add", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let caller_id = Self::get_as_string("caller_id", &params_map)?;
            let callee_id = Self::get_as_string("callee_id", &params_map)?;
            let handle = Self::get_as_string("handle", &params_map)?;

            let bridge = Bridge {
                caller_id,
                callee_id,
                handle,
            };
            container_call!(|c| c.add_bridge(bridge))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/bridge/remove", move |params| {
            let params_map = Self::unwrap_params_map(params)?;
            let caller_id = Self::get_as_string("caller_id", &params_map)?;
            let callee_id = Self::get_as_string("callee_id", &params_map)?;
            container_call!(|c| c.remove_bridge(&caller_id, &callee_id))?;
            Ok(json!({"success": true}))
        });

        self.io.add_method("admin/bridge/list", move |_params| {
            let bridges =
                container_call!(|c| Ok(c.config().bridges) as Result<Vec<Bridge>, String>)?;
            Ok(serde_json::to_value(bridges).map_err(|_| jsonrpc_core::Error::internal_error())?)
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

    fn create_call_str(method: &str, params: Option<&str>) -> String {
        json!({"jsonrpc": "2.0", "id": "0", "method": method, "params": params}).to_string()
    }

    /// checks that the response is a valid JSON string containing a `result` field which is stringified JSON
    ///
    fn unwrap_response_if_valid(response_str: &String) -> String {
        let result = &serde_json::from_str::<serde_json::Value>(response_str)
            .expect("Response not valid JSON")["result"];
        result.to_string()
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
        assert!(result.contains(r#""test-instance-1/greeter/hello""#));
        assert!(!result.contains(r#""test-instance-2//test""#));
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
        assert!(result.contains(r#""happ-store/greeter/hello""#));
        assert!(!result.contains(r#""test-instance-1//test""#));
    }

    /// The below test cannot be extented to test the other RPC methods due to the singleton design of the container
    /// It may be worth removing this test but I have included it as an example of testing the responses for the
    /// other rpc methods if this becomes possible in the future
    #[test]
    fn test_rpc_call_responses() {
        let (config, instances) = example_config_and_instances();
        let handler = ContainerApiBuilder::new()
            .with_instances(instances.clone())
            .with_instance_configs(config.instances)
            .with_admin_dna_functions()
            .spawn();

        let response_str = handler
            .handle_request_sync(&create_call_str("info/instances", None))
            .expect("Invalid call to handler");
        println!("{}", response_str);
        let result = unwrap_response_if_valid(&response_str);
        assert_eq!(
            result,
            r#"[{"id":"test-instance-1","dna":"bridge-callee","agent":"test-agent-1"}]"#
        );
    }
}
