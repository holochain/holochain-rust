use crate::{
    config::{Configuration, InterfaceConfiguration, InterfaceDriver, StorageConfiguration},
    error::HolochainInstanceError,
    Holochain,
};
use holochain_cas_implementations::{
    cas::{file::FilesystemStorage, memory::MemoryStorage},
    eav::{file::EavFileStorage, memory::EavMemoryStorage},
    path::create_path_if_not_exists,
};
use holochain_core::context::Context;
use holochain_core_types::{dna::Dna, error::HolochainError, json::JsonString};
use tempfile::tempdir;

use holochain_core::{logger::Logger, persister::SimplePersister};
use holochain_core_types::agent::AgentId;
use std::{
    clone::Clone,
    collections::HashMap,
    convert::TryFrom,
    fs::File,
    io::prelude::*,
    sync::{Arc, Mutex, RwLock},
    thread,
};

use interface::{ContainerApiDispatcher, InstanceMap, Interface};
use interface_impls;
use holochain_net::p2p_config::P2pConfig;
/// Main representation of the container.
/// Holds a `HashMap` of Holochain instances referenced by ID.

/// A primary point in this struct is
/// `load_config(&mut self, config: &Configuration) -> Result<(), String>`
/// which takes a `config::Configuration` struct and tries to instantiate all configured instances.
/// While doing so it has to load DNA files referenced in the configuration.
/// In order to not bind this code to the assumption that there is a filesystem
/// and also enable easier testing, a DnaLoader ()which is a closure that returns a
/// Dna object for a given path string) has to be injected on creation.
pub struct Container {
    pub instances: InstanceMap,
    config: Configuration,
    interface_threads: HashMap<String, InterfaceThreadHandle>,
    pub dna_loader: DnaLoader,
}

type InterfaceThreadHandle = thread::JoinHandle<Result<(), String>>;
type DnaLoader = Arc<Box<FnMut(&String) -> Result<Dna, HolochainError> + Send>>;

pub static DEFAULT_NETWORK_CONFIG: &'static str = P2pConfig::DEFAULT_MOCK_CONFIG;

impl Container {
    /// Creates a new instance with the default DnaLoader that actually loads files.
    pub fn with_config(config: Configuration) -> Self {
        Container {
            instances: HashMap::new(),
            interface_threads: HashMap::new(),
            config,
            dna_loader: Arc::new(Box::new(Self::load_dna)),
        }
    }

    pub fn start_all_interfaces(&mut self) {
        self.interface_threads = self
            .config
            .interfaces
            .iter()
            .map(|ic| (ic.id.clone(), self.spawn_interface_thread(ic.clone())))
            .collect()
    }

    pub fn start_interface_by_id(&mut self, id: String) -> Result<(), String> {
        self.config
            .interface_by_id(&id)
            .ok_or(format!("Interface does not exist: {}", id))
            .and_then(|config| self.start_interface(&config))
    }

    /// Starts all instances
    pub fn start_all_instances(&mut self) -> Result<(), HolochainInstanceError> {
        self.instances
            .iter_mut()
            .map(|(id, hc)| {
                println!("Starting instance \"{}\"...", id);
                hc.write().unwrap().start()
            })
            .collect::<Result<Vec<()>, _>>()
            .map(|_| ())
    }

    /// Stops all instances
    pub fn stop_all_instances(&mut self) -> Result<(), HolochainInstanceError> {
        self.instances
            .iter_mut()
            .map(|(id, hc)| {
                println!("Stopping instance \"{}\"...", id);
                hc.write().unwrap().stop()
            })
            .collect::<Result<Vec<()>, _>>()
            .map(|_| ())
    }

    /// Stop and clear all instances
    pub fn shutdown(&mut self) -> Result<(), HolochainInstanceError> {
        self.stop_all_instances()?;
        self.instances = HashMap::new();
        Ok(())
    }

    /// Tries to create all instances configured in the given Configuration object.
    /// Calls `Configuration::check_consistency()` first and clears `self.instances`.
    pub fn load_config(&mut self, config: &Configuration) -> Result<(), String> {
        let _ = config.check_consistency()?;
        self.shutdown().map_err(|e| e.to_string())?;
        let default_network = DEFAULT_NETWORK_CONFIG.to_string();
        let id_instance_pairs: Vec<_> = config
            .instance_ids()
            .clone()
            .into_iter()
            .map(|id| {
                (
                    id.clone(),
                    instantiate_from_config(&id, config, &mut self.dna_loader, &default_network),
                )
            })
            .collect();

        let errors: Vec<_> = id_instance_pairs
            .into_iter()
            .filter_map(|(id, maybe_holochain)| match maybe_holochain {
                Ok(holochain) => {
                    self.instances
                        .insert(id.clone(), Arc::new(RwLock::new(holochain)));
                    None
                }
                Err(error) => Some(format!(
                    "Error while trying to create instance \"{}\": {}",
                    id, error
                )),
            })
            .collect();

        if errors.len() == 0 {
            Ok(())
        } else {
            Err(errors.iter().nth(0).unwrap().clone())
        }
    }

    fn start_interface(&mut self, config: &InterfaceConfiguration) -> Result<(), String> {
        if self.interface_threads.contains_key(&config.id) {
            return Err(format!("Interface {} already started!", config.id));
        }
        let handle = self.spawn_interface_thread(config.clone());
        self.interface_threads.insert(config.id.clone(), handle);
        Ok(())
    }

    /// Default DnaLoader that actually reads files from the filesystem
    fn load_dna(file: &String) -> Result<Dna, HolochainError> {
        let mut f = File::open(file)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        Dna::try_from(JsonString::from(contents))
    }

    fn make_dispatcher(&self, interface_config: &InterfaceConfiguration) -> ContainerApiDispatcher {
        let instance_ids: Vec<String> = interface_config
            .instances
            .iter()
            .map(|i| i.id.clone())
            .collect();
        let instance_subset: InstanceMap = self
            .instances
            .iter()
            .filter(|(id, _)| instance_ids.contains(&id))
            .map(|(id, val)| (id.clone(), val.clone()))
            .collect();
        ContainerApiDispatcher::new(&self.config, instance_subset)
    }

    fn spawn_interface_thread(
        &self,
        interface_config: InterfaceConfiguration,
    ) -> InterfaceThreadHandle {
        let dispatcher = self.make_dispatcher(&interface_config);
        thread::spawn(move || {
            let iface = make_interface(&interface_config);
            iface.run(dispatcher)
        })
    }
}

impl<'a> TryFrom<&'a Configuration> for Container {
    type Error = HolochainError;
    fn try_from(config: &'a Configuration) -> Result<Self, Self::Error> {
        let mut container = Container::with_config((*config).clone());
        container
            .load_config(config)
            .map_err(|string| HolochainError::ConfigError(string))?;
        Ok(container)
    }
}

/// This can eventually be dependency injected for third party Interface definitions
fn make_interface(
    interface_config: &InterfaceConfiguration,
) -> Box<Interface<ContainerApiDispatcher>> {
    match interface_config.driver {
        InterfaceDriver::Websocket { port } => {
            Box::new(interface_impls::websocket::WebsocketInterface::new(port))
        }
        _ => unimplemented!(),
    }
}

/// Creates one specific Holochain instance from a given Configuration,
/// id string and DnaLoader.
pub fn instantiate_from_config(
    id: &String,
    config: &Configuration,
    dna_loader: &mut DnaLoader,
    default_network_config: &String,
) -> Result<Holochain, String> {
    let _ = config.check_consistency()?;

    config
        .instance_by_id(&id)
        .ok_or(String::from("Instance not found in config"))
        .and_then(|instance_config| {
            let agent_config = config.agent_by_id(&instance_config.agent).unwrap();
            let dna_config = config.dna_by_id(&instance_config.dna).unwrap();
            let dna = Arc::get_mut(dna_loader).unwrap()(&dna_config.file).map_err(|_| {
                HolochainError::ConfigError(format!(
                    "Could not load DNA file \"{}\"",
                    dna_config.file
                ))
            })?;

            let network_config = instance_config
                .network
                .unwrap_or(default_network_config.to_owned())
                .into();

            let context: Context = match instance_config.storage {
                StorageConfiguration::File { path } => {
                    create_file_context(&agent_config.id, &path, network_config)
                        .map_err(|hc_err| format!("Error creating context: {}", hc_err.to_string()))
                }
                StorageConfiguration::Memory => {
                    create_memory_context(&agent_config.id, network_config)
                        .map_err(|hc_err| format!("Error creating context: {}", hc_err.to_string()))
                }
            }?;

            Holochain::new(dna, Arc::new(context)).map_err(|hc_err| hc_err.to_string())
        })
}

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

fn create_memory_context(
    _: &String,
    network_config: JsonString,
) -> Result<Context, HolochainError> {
    let agent = AgentId::generate_fake("c+bob");
    let tempdir = tempdir().unwrap();
    let file_storage = Arc::new(RwLock::new(
        FilesystemStorage::new(tempdir.path().to_str().unwrap()).unwrap(),
    ));

    Context::new(
        agent,
        Arc::new(Mutex::new(NullLogger {})),
        Arc::new(Mutex::new(SimplePersister::new(file_storage.clone()))),
        Arc::new(RwLock::new(MemoryStorage::new())),
        Arc::new(RwLock::new(EavMemoryStorage::new())),
        network_config,
    )
}

fn create_file_context(
    _: &String,
    path: &String,
    network_config: JsonString,
) -> Result<Context, HolochainError> {
    let agent = AgentId::generate_fake("c+bob");
    let cas_path = format!("{}/cas", path);
    let eav_path = format!("{}/eav", path);
    create_path_if_not_exists(&cas_path)?;
    create_path_if_not_exists(&eav_path)?;

    let file_storage = Arc::new(RwLock::new(FilesystemStorage::new(&cas_path)?));

    Context::new(
        agent,
        Arc::new(Mutex::new(NullLogger {})),
        Arc::new(Mutex::new(SimplePersister::new(file_storage.clone()))),
        file_storage.clone(),
        Arc::new(RwLock::new(EavFileStorage::new(eav_path)?)),
        network_config,
    )
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::config::load_configuration;
    use std::{fs::File, io::Write};

    use tempfile::tempdir;

    pub fn test_dna_loader() -> DnaLoader {
        let loader = Box::new(|_path: &String| {
            Ok(Dna::try_from(JsonString::from(example_dna_string())).unwrap())
        }) as Box<FnMut(&String) -> Result<Dna, HolochainError> + Send>;
        Arc::new(loader)
    }

    pub fn test_toml() -> String {
        r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester"
    key_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app_spec.hcpkg"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app spec instance"
    dna = "app spec rust"
    agent = "test agent"
    [instances.logger]
    type = "simple"
    file = "app_spec.log"
    [instances.storage]
    type = "memory"

    [[interfaces]]
    id = "app spec interface"
    [interfaces.driver]
    type = "websocket"
    port = 8888
    [[interfaces.instances]]
    id = "app spec instance"
    "#
        .to_string()
    }

    pub fn example_dna_string() -> String {
        r#"{
                "name": "my dna",
                "description": "",
                "version": "",
                "uuid": "00000000-0000-0000-0000-000000000001",
                "dna_spec_version": "2.0",
                "properties": {},
                "zomes": {
                    "": {
                        "description": "",
                        "config": {
                            "error_handling": "throw-errors"
                        },
                        "entry_types": {
                            "": {
                                "description": "",
                                "sharing": "public"
                            }
                        },
                        "capabilities": {
                            "test": {
                                "capability": {
                                    "membrane": "public"
                                },
                                "functions": [
                                    {
                                        "name": "test",
                       "inputs" : [
                            {
                                "name": "post",
                                "type": "string"
                            }
                        ],
                        "outputs" : [
                            {
                                "name": "hash",
                                "type": "string"
                            }
                        ]
                                    }
                                ]
                            }
                        },
                        "code": {
                            "code": "AAECAw=="
                        }
                    }
                }
            }"#
        .to_string()
    }

    #[test]
    fn test_instantiate_from_config() {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();
        let default_network = DEFAULT_NETWORK_CONFIG.to_string();
        let maybe_holochain = instantiate_from_config(
            &"app spec instance".to_string(),
            &config,
            &mut test_dna_loader(),
            &default_network,
        );

        assert_eq!(maybe_holochain.err(), None);
    }

    #[test]
    fn test_default_dna_loader() {
        let tempdir = tempdir().unwrap();
        let file_path = tempdir.path().join("test.dna.json");
        let mut tmp_file = File::create(file_path.clone()).unwrap();
        writeln!(tmp_file, "{}", example_dna_string()).unwrap();
        match Container::load_dna(&file_path.into_os_string().into_string().unwrap()) {
            Ok(dna) => {
                assert_eq!(dna.name, "my dna");
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_container_load_config() {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();

        // TODO: redundant, see https://github.com/holochain/holochain-rust/issues/674
        let mut container = Container::with_config(config.clone());
        container.dna_loader = test_dna_loader();

        container.load_config(&config).unwrap();
        assert_eq!(container.instances.len(), 1);

        container.start_all_instances().unwrap();
        container.start_all_interfaces();
        container.stop_all_instances().unwrap();
    }

    #[test]
    fn test_container_try_from_configuration() {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();

        let maybe_container = Container::try_from(&config);

        assert!(maybe_container.is_err());
        assert_eq!(
            maybe_container.err().unwrap(),
            HolochainError::ConfigError(
                "Error while trying to create instance \"app spec instance\": Could not load DNA file \"app_spec.hcpkg\"".to_string()
            )
        );
    }

    #[test]
    fn test_rpc_info_instances() {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();

        // TODO: redundant, see https://github.com/holochain/holochain-rust/issues/674
        let mut container = Container::with_config(config.clone());
        container.dna_loader = test_dna_loader();
        container.load_config(&config).unwrap();

        let instance_config = &config.interfaces[0];
        let dispatcher = container.make_dispatcher(&instance_config);
        let io = dispatcher.io;

        let request = r#"{"jsonrpc": "2.0", "method": "info/instances", "params": null, "id": 1}"#;
        let response = r#"{"jsonrpc":"2.0","result":"{\"app spec instance\":{\"id\":\"app spec instance\",\"dna\":\"app spec rust\",\"agent\":\"test agent\",\"logger\":{\"type\":\"simple\",\"file\":\"app_spec.log\"},\"storage\":{\"type\":\"memory\"},\"network\":null}}","id":1}"#;

        assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
    }
}
