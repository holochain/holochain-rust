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
use holochain_core::{
    context::Context, logger::Logger, persister::SimplePersister, signal::Signal,
};
use holochain_core_types::{dna::Dna, error::HolochainError, json::JsonString};
use tempfile::tempdir;

use holochain_core_types::agent::AgentId;
use std::{
    clone::Clone,
    collections::HashMap,
    convert::TryFrom,
    fs::File,
    io::prelude::*,
    sync::{mpsc::SyncSender, Arc, Mutex, RwLock},
    thread,
};

use holochain_net::p2p_config::P2pConfig;
use interface::{ContainerApiDispatcher, InstanceMap, Interface};
use interface_impls;
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
    dna_loader: DnaLoader,
    signal_tx: Option<SignalSender>,
}

type SignalSender = SyncSender<Signal>;
type InterfaceThreadHandle = thread::JoinHandle<Result<(), String>>;
type DnaLoader = Arc<Box<FnMut(&String) -> Result<Dna, HolochainError> + Send>>;

pub static DEFAULT_NETWORK_CONFIG: &'static str = P2pConfig::DEFAULT_MOCK_CONFIG;

impl Container {
    /// Creates a new instance with the default DnaLoader that actually loads files.
    pub fn from_config(config: Configuration) -> Self {
        Container {
            instances: HashMap::new(),
            interface_threads: HashMap::new(),
            config,
            dna_loader: Arc::new(Box::new(Self::load_dna)),
            signal_tx: None,
        }
    }

    pub fn with_signal_channel(mut self, signal_tx: SyncSender<Signal>) -> Self {
        if !self.instances.is_empty() {
            panic!("Cannot set a signal channel after having run load_config()");
        }
        self.signal_tx = Some(signal_tx);
        self
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
        println!("NUM INSTANCES: {}", self.instances.len());
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

    /// Directly access an instance in this container, useful for e.g. testing frameworks
    pub fn get_instance_by_id(&self, id: &str) -> Option<Arc<RwLock<Holochain>>> {
        self.instances.get(id).map(|hc| hc.clone())
    }

    /// Stop and clear all instances
    pub fn shutdown(&mut self) -> Result<(), HolochainInstanceError> {
        self.stop_all_instances()?;
        // @TODO: also stop all interfaces
        self.instances = HashMap::new();
        Ok(())
    }

    /// Tries to create all instances configured in the given Configuration object.
    /// Calls `Configuration::check_consistency()` first and clears `self.instances`.
    /// @TODO: clean up the container creation process to prevent loading config before proper setup,
    ///        especially regarding the signal handler.
    ///        (see https://github.com/holochain/holochain-rust/issues/739)
    pub fn load_config(&mut self) -> Result<(), String> {
        let _ = self.config.check_consistency()?;
        self.shutdown().map_err(|e| e.to_string())?;
        let config = self.config.clone();
        let default_network = DEFAULT_NETWORK_CONFIG.to_string();
        let mut instances = HashMap::new();

        let errors: Vec<_> = config
            .instance_ids()
            .clone()
            .into_iter()
            .map(|id| {
                (
                    id.clone(),
                    instantiate_from_config(
                        &id,
                        &config,
                        &mut self.dna_loader,
                        &default_network,
                        self.signal_tx.clone(),
                    ),
                )
            })
            .filter_map(|(id, maybe_holochain)| match maybe_holochain {
                Ok(holochain) => {
                    instances.insert(id.clone(), Arc::new(RwLock::new(holochain)));
                    None
                }
                Err(error) => Some(format!(
                    "Error while trying to create instance \"{}\": {}",
                    id, error
                )),
            })
            .collect();

        if errors.len() == 0 {
            self.instances = instances;
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
        let mut container = Container::from_config((*config).clone());
        container
            .load_config()
            .map_err(|string| HolochainError::ConfigError(string))?;
        Ok(container)
    }
}

/// This can eventually be dependency injected for third party Interface definitions
fn make_interface(
    interface_config: &InterfaceConfiguration,
) -> Box<Interface<ContainerApiDispatcher>> {
    use interface_impls::websocket::WebsocketInterface;
    match interface_config.driver {
        InterfaceDriver::Websocket { port } => Box::new(WebsocketInterface::new(port)),
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
    signal_tx: Option<SignalSender>,
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
            match signal_tx {
                Some(signal_tx) => {
                    Holochain::new_with_signals(dna, Arc::new(context), signal_tx, |_| true)
                }
                None => Holochain::new(dna, Arc::new(context)),
            }
            .map_err(|hc_err| hc_err.to_string())
        })
}

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

fn create_memory_context(
    agent_name: &String,
    network_config: JsonString,
) -> Result<Context, HolochainError> {
    let agent = AgentId::generate_fake(agent_name);
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

    use holochain_core::signal::signal_channel;
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
    id = "test-agent-1"
    name = "Holo Tester 1"
    key_file = "holo_tester.key"

    [[agents]]
    id = "test-agent-2"
    name = "Holo Tester 2"
    key_file = "holo_tester.key"

    [[dnas]]
    id = "test-dna"
    file = "app_spec.hcpkg"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "test-instance-1"
    dna = "test-dna"
    agent = "test-agent-1"
    [instances.logger]
    type = "simple"
    file = "app_spec.log"
    [instances.storage]
    type = "memory"

    [[instances]]
    id = "test-instance-2"
    dna = "test-dna"
    agent = "test-agent-2"
    [instances.logger]
    type = "simple"
    file = "app_spec.log"
    [instances.storage]
    type = "memory"

    [[interfaces]]
    id = "test-interface"
    [interfaces.driver]
    type = "websocket"
    port = 8888
    [[interfaces.instances]]
    id = "test-instance-1"
    [[interfaces.instances]]
    id = "test-instance-2"
    "#
        .to_string()
    }

    fn test_container() -> Container {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();
        let mut container = Container::from_config(config.clone());
        container.dna_loader = test_dna_loader();
        container.load_config().unwrap();
        container
    }

    fn test_container_with_signals(signal_tx: SignalSender) -> Container {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();
        let mut container = Container::from_config(config.clone()).with_signal_channel(signal_tx);
        container.dna_loader = test_dna_loader();
        container.load_config().unwrap();
        container
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
        let (tx, _) = signal_channel();
        let maybe_holochain = instantiate_from_config(
            &"test-instance-1".to_string(),
            &config,
            &mut test_dna_loader(),
            &default_network,
            Some(tx),
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
        let mut container = test_container();
        assert_eq!(container.instances.len(), 2);

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
                "Error while trying to create instance \"test-instance-1\": Could not load DNA file \"app_spec.hcpkg\"".to_string()
            )
        );
    }

    #[test]
    fn test_rpc_info_instances() {
        let container = test_container();
        let interface_config = &container.config.interfaces[0];
        let dispatcher = container.make_dispatcher(&interface_config);
        let io = dispatcher.io;

        let request = r#"{"jsonrpc": "2.0", "method": "info/instances", "params": null, "id": 1}"#;
        let response = io
            .handle_request_sync(request)
            .expect("No response returned for info/instances");
        assert!(response.contains("test-instance-1"));
        assert!(response.contains("test-instance-2"));
    }

    #[test]
    fn container_signal_handler() {
        use holochain_core::action::Action;
        let (signal_tx, signal_rx) = signal_channel();
        let _container = test_container_with_signals(signal_tx);

        test_utils::expect_action(&signal_rx, |action| match action {
            Action::InitApplication(_) => true,
            _ => false,
        })
        .unwrap();

        // expect one InitNetwork for each instance

        test_utils::expect_action(&signal_rx, |action| match action {
            Action::InitNetwork(_) => true,
            _ => false,
        })
        .unwrap();

        test_utils::expect_action(&signal_rx, |action| match action {
            Action::InitNetwork(_) => true,
            _ => false,
        })
        .unwrap();
    }

}
