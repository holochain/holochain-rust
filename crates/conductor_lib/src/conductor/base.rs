use crate::{
    conductor::broadcaster::Broadcaster,
    config::{
        serialize_configuration, Configuration, InterfaceConfiguration, InterfaceDriver,
        NetworkConfig, StorageConfiguration,
    },
    context_builder::ContextBuilder,
    dpki_instance::DpkiInstance,
    error::HolochainInstanceError,
    keystore::{Keystore, PRIMARY_KEYBUNDLE_ID},
    Holochain,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use holochain_common::paths::DNA_EXTENSION;
use holochain_core::{logger::Logger, signal::Signal};
use holochain_core_types::{
    agent::AgentId,
    dna::Dna,
    error::{HcResult, HolochainError},
    sync::{HcMutex as Mutex, HcRwLock as RwLock},
};
use key_loaders::test_keystore;

use holochain_json_api::json::JsonString;
use holochain_persistence_api::{cas::content::AddressableContent, hash::HashString};

use holochain_dpki::{key_bundle::KeyBundle, password_encryption::PwHashConfig};
use holochain_logging::{rule::RuleFilter, FastLogger, FastLoggerBuilder};
use jsonrpc_ws_server::jsonrpc_core::IoHandler;
use std::{
    clone::Clone,
    collections::HashMap,
    convert::TryFrom,
    fs::{self, File},
    io::prelude::*,
    option::NoneError,
    path::PathBuf,
    sync::Arc,
    thread,
    time::Duration,
};

use boolinator::Boolinator;
#[cfg(unix)]
use conductor::passphrase_manager::PassphraseServiceUnixSocket;
use conductor::passphrase_manager::{
    PassphraseManager, PassphraseService, PassphraseServiceCmd, PassphraseServiceMock,
};
use config::{AgentConfiguration, PassphraseServiceConfig};
use holochain_core_types::dna::bridges::BridgePresence;
use holochain_net::{
    connection::net_connection::NetHandler,
    ipc::spawn::{ipc_spawn, SpawnResult},
    p2p_config::{BackendConfig, P2pBackendKind, P2pConfig},
    p2p_network::P2pNetwork,
};
use interface::{ConductorApiBuilder, InstanceMap, Interface};
use signal_wrapper::SignalWrapper;
use static_file_server::ConductorStaticFileServer;
use static_server_impls::NickelStaticServer as StaticServer;

lazy_static! {
    /// This is a global and mutable Conductor singleton.
    /// (Ok, not really. I've made Conductor::from_config public again so holochain_nodejs
    /// is not forced to use Conductor as a singleton so we don't run into problems with
    /// tests affecting each other. The consequence is that Rustc can't help us in enforcing
    /// the conductor to be singleton otherwise. The only point this is important anyway is in
    /// the interfaces. That code needs this static variable to be set in order to be able to
    /// call ConductorAdmin functions.)
    /// In order to call from interface threads Conductor admin functions that change
    /// the config and hence mutate the Conductor, we need something that owns the Conductor
    /// and is accessible from everywhere (esp. those conductor interface method closures
    /// in interface.rs).
    pub static ref CONDUCTOR: Arc<Mutex<Option<Conductor>>> = Arc::new(Mutex::new(None));
}

/// Conductor constructor that makes sure the Conductor instance object is mounted
/// in above static CONDUCTOR.
/// It replaces any Conductor instance that was mounted before to CONDUCTOR with a new one
/// create from the given configuration.
pub fn mount_conductor_from_config(config: Configuration) {
    let conductor = Conductor::from_config(config);
    CONDUCTOR.lock().unwrap().replace(conductor);
}

/// Main representation of the conductor.
/// Holds a `HashMap` of Holochain instances referenced by ID.
/// A primary point in this struct is
/// `load_config(&mut self, config: &Configuration) -> Result<(), String>`
/// which takes a `config::Configuration` struct and tries to instantiate all configured instances.
/// While doing so it has to load DNA files referenced in the configuration.
/// In order to not bind this code to the assumption that there is a filesystem
/// and also enable easier testing, a DnaLoader ()which is a closure that returns a
/// Dna object for a given path string) has to be injected on creation.
pub struct Conductor {
    pub(in crate::conductor) instances: InstanceMap,
    instance_signal_receivers: Arc<RwLock<HashMap<String, Receiver<Signal>>>>,
    agent_keys: HashMap<String, Arc<Mutex<Keystore>>>,
    pub(in crate::conductor) config: Configuration,
    pub(in crate::conductor) static_servers: HashMap<String, StaticServer>,
    pub(in crate::conductor) interface_threads: HashMap<String, Sender<()>>,
    pub(in crate::conductor) interface_broadcasters: Arc<RwLock<HashMap<String, Broadcaster>>>,
    signal_multiplexer_kill_switch: Option<Sender<()>>,
    pub key_loader: KeyLoader,
    pub(in crate::conductor) dna_loader: DnaLoader,
    pub(in crate::conductor) ui_dir_copier: UiDirCopier,
    signal_tx: Option<SignalSender>,
    logger: FastLogger,
    p2p_config: Option<P2pConfig>,
    network_spawn: Option<SpawnResult>,
    pub passphrase_manager: Arc<PassphraseManager>,
    pub hash_config: Option<PwHashConfig>, // currently this has to be pub for testing.  would like to remove
    // TODO: remove this when n3h gets deprecated
    n3h_keepalive_network: Option<P2pNetwork>, // hack needed so that n3h process stays alive even if all instances get shutdown.
}

impl Drop for Conductor {
    fn drop(&mut self) {
        if let Some(ref mut network_spawn) = self.network_spawn {
            if let Some(mut kill) = network_spawn.kill.take() {
                kill();
            }
        }

        self.shutdown()
            .unwrap_or_else(|err| println!("Error during shutdown, continuing anyway: {:?}", err));

        // Flushing the logger's buffer writer
        self.logger.flush();
        // Do not shut down the logging thread if there is multiple concurrent conductor thread
        // like during unit testing because they all use the same registered logger
        // self.logger.shutdown();

        if let Some(mut network) = self.n3h_keepalive_network.take() {
            network.stop()
        }
    }
}

type SignalSender = Sender<Signal>;
pub type KeyLoader = Arc<
    Box<
        dyn FnMut(
                &PathBuf,
                Arc<PassphraseManager>,
            ) -> Result<Keystore, HolochainError>
            + Send
            + Sync,
    >,
>;
pub type DnaLoader = Arc<Box<dyn FnMut(&PathBuf) -> Result<Dna, HolochainError> + Send + Sync>>;
pub type UiDirCopier =
    Arc<Box<dyn FnMut(&PathBuf, &PathBuf) -> Result<(), HolochainError> + Send + Sync>>;

/// preparing for having conductor notifiers go to one of the log streams
pub fn notify(msg: String) {
    println!("{}", msg);
}

impl Conductor {
    pub fn from_config(config: Configuration) -> Self {
        lib3h_sodium::check_init();
        let _rules = config.logger.rules.clone();
        let mut logger_builder = FastLoggerBuilder::new();
        logger_builder.set_level_from_str(&config.logger.logger_level.as_str());

        for rule in config.logger.rules.rules.iter() {
            logger_builder.add_rule_filter(RuleFilter::new(
                rule.pattern.as_str(),
                rule.exclude,
                rule.color.as_ref().unwrap_or(&String::default()).as_str(),
            ));
        }

        let logger = logger_builder
            .build()
            .expect("Fail to instanciate the logging factory.");

        if config.ui_bundles.len() > 0 || config.ui_interfaces.len() > 0 {
            println!();
            println!("{}", std::iter::repeat("!").take(20).collect::<String>());
            println!("DEPRECATION WARNING - Hosting a static UI via the conductor will not be supported in future releases");
            println!("{}", std::iter::repeat("!").take(20).collect::<String>());
            println!();
        }

        let passphrase_service: Arc<Mutex<dyn PassphraseService + Send>> =
            if let PassphraseServiceConfig::UnixSocket { path } = config.passphrase_service.clone()
            {
                #[cfg(not(unix))]
                let _ = path;
                #[cfg(not(unix))]
                panic!("Unix domain sockets are not available on non-Unix systems. Can't create a PassphraseServiceUnixSocket.");

                #[cfg(unix)]
                Arc::new(Mutex::new(PassphraseServiceUnixSocket::new(path)))
            } else {
                match config.passphrase_service.clone() {
                    PassphraseServiceConfig::Cmd => Arc::new(Mutex::new(PassphraseServiceCmd {})),
                    PassphraseServiceConfig::Mock { passphrase } => {
                        Arc::new(Mutex::new(PassphraseServiceMock { passphrase }))
                    }
                    _ => unreachable!(),
                }
            };

        Conductor {
            instances: HashMap::new(),
            instance_signal_receivers: Arc::new(RwLock::new(HashMap::new())),
            agent_keys: HashMap::new(),
            interface_threads: HashMap::new(),
            static_servers: HashMap::new(),
            interface_broadcasters: Arc::new(RwLock::new(HashMap::new())),
            signal_multiplexer_kill_switch: None,
            config,
            key_loader: Arc::new(Box::new(Self::load_key)),
            dna_loader: Arc::new(Box::new(Self::load_dna)),
            ui_dir_copier: Arc::new(Box::new(Self::copy_ui_dir)),
            signal_tx: None,
            logger,
            p2p_config: None,
            network_spawn: None,
            passphrase_manager: Arc::new(PassphraseManager::new(passphrase_service)),
            hash_config: None,
            n3h_keepalive_network: None,
        }
    }

    pub fn add_agent_keystore(&mut self, agent_id: String, keystore: Keystore) {
        self.agent_keys
            .insert(agent_id, Arc::new(Mutex::new(keystore)));
    }

    pub fn with_signal_channel(mut self, signal_tx: Sender<Signal>) -> Self {
        // TODO: clean up the conductor creation process to prevent loading config before proper setup,
        // especially regarding the signal handler.
        // (see https://github.com/holochain/holochain-rust/issues/739)
        if !self.instances.is_empty() {
            panic!("Cannot set a signal channel after having run from_config()");
        }
        self.signal_tx = Some(signal_tx);
        self
    }

    pub fn p2p_bindings(&self) -> Option<Vec<String>> {
        self.network_spawn
            .as_ref()
            .map(|spawn| spawn.p2p_bindings.clone())
    }

    pub fn config(&self) -> Configuration {
        self.config.clone()
    }

    /// Starts a new thread which monitors each instance's signal channel and pushes signals out
    /// all interfaces the according instance is part of.
    pub fn start_signal_multiplexer(&mut self) -> thread::JoinHandle<()> {
        self.stop_signal_multiplexer();
        let broadcasters = self.interface_broadcasters.clone();
        let instance_signal_receivers = self.instance_signal_receivers.clone();
        let signal_tx = self.signal_tx.clone();
        let config = self.config.clone();
        let (kill_switch_tx, kill_switch_rx) = unbounded();
        self.signal_multiplexer_kill_switch = Some(kill_switch_tx);

        debug!("starting signal loop");
        thread::Builder::new()
            .name("signal_multiplexer".to_string())
            .spawn(move || loop {
                {
                    for (instance_id, receiver) in instance_signal_receivers.read().unwrap().iter()
                    {
                        if let Ok(signal) = receiver.try_recv() {
                            signal_tx.clone().map(|s| s.send(signal.clone()));
                            let broadcasters = broadcasters.read().unwrap();
                            let interfaces_with_instance: Vec<&InterfaceConfiguration> =
                                match signal {
                                    // Send internal signals only to admin interfaces, if signals.trace is set:
                                    Signal::Trace(_) => {
                                        if config.signals.trace {
                                            config
                                                .interfaces
                                                .iter()
                                                .filter(|interface_config| interface_config.admin)
                                                .collect()
                                        } else {
                                            Vec::new()
                                        }
                                    }

                                    // Send internal signals only to admin interfaces, if signals.consistency is set:
                                    Signal::Consistency(_) => {
                                        if config.signals.consistency {
                                            config
                                                .interfaces
                                                .iter()
                                                .filter(|interface_config| interface_config.admin)
                                                .collect()
                                        } else {
                                            Vec::new()
                                        }
                                    }

                                    // Pass through user-defined  signals to the according interfaces
                                    // in which the source instance is exposed:
                                    Signal::User(_) => {
                                        println!(
                                            "SIGNAL for instance[{}]: {:?}",
                                            instance_id, signal
                                        );
                                        let interfaces = config
                                            .interfaces
                                            .iter()
                                            .filter(|interface_config| {
                                                interface_config
                                                    .instances
                                                    .iter()
                                                    .any(|instance| instance.id == *instance_id)
                                            })
                                            .collect();
                                        println!("INTERFACEs for SIGNAL: {:?}", interfaces);
                                        interfaces
                                    }
                                };

                            for interface in interfaces_with_instance {
                                if let Some(broadcaster) = broadcasters.get(&interface.id) {
                                    if let Err(error) = broadcaster.send(SignalWrapper {
                                        signal: signal.clone(),
                                        instance_id: instance_id.clone(),
                                    }) {
                                        notify(error.to_string());
                                    }
                                };
                            }
                        }
                    }
                }
                if kill_switch_rx.try_recv().is_ok() {
                    break;
                }
                thread::sleep(Duration::from_millis(1));
            })
            .expect("Must be able to spawn thread")
    }

    pub fn stop_signal_multiplexer(&self) {
        self.signal_multiplexer_kill_switch
            .as_ref()
            .map(|kill_switch| kill_switch.send(()));
    }

    pub fn start_all_interfaces(&mut self) {
        self.interface_threads = self
            .config
            .interfaces
            .iter()
            .map(|ic| (ic.id.clone(), self.spawn_interface_thread(ic.clone())))
            .collect()
    }

    pub fn stop_all_interfaces(&mut self) {
        for (id, kill_switch) in self.interface_threads.iter() {
            notify(format!("Stopping interface {}", id));
            kill_switch.send(()).unwrap_or_else(|err| {
                let message = format!("Error stopping interface: {}", err);
                notify(message.clone());
            });
        }
    }

    pub fn stop_interface_by_id(&mut self, id: &String) -> Result<(), HolochainError> {
        {
            let kill_switch = self.interface_threads.get(id).ok_or_else(|| {
                HolochainError::ErrorGeneric(format!("Interface {} not found.", id))
            })?;
            notify(format!("Stopping interface {}", id));
            kill_switch.send(()).map_err(|err| {
                let message = format!("Error stopping interface: {}", err);
                notify(message.clone());
                HolochainError::ErrorGeneric(message)
            })?;
        }
        self.interface_threads.remove(id);
        Ok(())
    }

    pub fn start_interface_by_id(&mut self, id: &String) -> Result<(), String> {
        notify(format!("Start interface by id: {}", id));
        self.config
            .interface_by_id(id)
            .ok_or_else(|| format!("Interface does not exist: {}", id))
            .and_then(|config| self.start_interface(&config))
    }

    pub fn start_all_static_servers(&mut self) -> Result<(), String> {
        notify("Starting all servers".into());
        self.static_servers.iter_mut().for_each(|(id, server)| {
            notify(format!("Starting server \"{}|\"", id));
            server
                .start()
                .unwrap_or_else(|_| panic!("Couldn't start server {}", id));
            notify(format!("Server started for \"{}\"", id))
        });
        Ok(())
    }

    pub fn start_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError> {
        let mut instance = self.instances.get(id)?.write().unwrap();
        notify(format!("Starting instance \"{}\"...", id));

        // Get instance DNA so we can read out required bridge definitions:
        let dna =
            instance
                .state()?
                .nucleus()
                .dna()
                .ok_or(HolochainInstanceError::InternalFailure(
                    HolochainError::DnaMissing,
                ))?;

        // Make sure required bridges are configured and started:
        for zome in dna.zomes.values() {
            for bridge in zome.bridges.iter() {
                if bridge.presence == BridgePresence::Required {
                    let handle = bridge.handle.clone();
                    let bridge_config = self
                        .config
                        .bridges
                        .iter()
                        .find(|b| b.handle == handle)
                        .ok_or_else(|| {
                            HolochainInstanceError::RequiredBridgeMissing(handle.clone())
                        })?;
                    self.instances
                        .get(&bridge_config.callee_id)
                        .ok_or_else(|| {
                            HolochainInstanceError::RequiredBridgeMissing(handle.clone())
                        })?
                        .read()
                        .unwrap()
                        .active()
                        .ok_or_else(|| HolochainInstanceError::RequiredBridgeMissing(handle))?;
                }
            }
        }
        instance.start()
    }

    pub fn stop_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError> {
        let instance = self.instances.get(id)?;
        notify(format!("Stopping instance \"{}\"...", id));
        instance.write().unwrap().stop()
    }

    /// Starts all instances
    pub fn start_all_instances(&mut self) -> Result<(), HolochainInstanceError> {
        notify("Start all instances".to_string());
        self.config
            .instances
            .iter()
            .map(|instance_config| instance_config.id.clone())
            .collect::<Vec<String>>()
            .iter()
            .map(|id| {
                let start_result = self.start_instance(&id);
                if Err(HolochainInstanceError::InstanceAlreadyActive) == start_result {
                    Ok(())
                } else {
                    start_result
                }
            })
            .collect::<Result<Vec<()>, _>>()
            .map(|_| ())
    }

    /// Starts dpki_happ instances
    pub fn start_dpki_instance(&mut self) -> Result<(), HolochainInstanceError> {
        let dpki_instance_id = &self.dpki_instance_id().unwrap();
        let mut instance = self
            .instantiate_from_config(dpki_instance_id)
            .map_err(|err| {
                HolochainInstanceError::InternalFailure(HolochainError::ErrorGeneric(err))
            })?;
        instance.start()?;
        self.instances.insert(
            dpki_instance_id.to_string(),
            Arc::new(RwLock::new(instance)),
        );
        Ok(())
    }

    /// Stops all instances
    pub fn stop_all_instances(&mut self) -> Result<(), HolochainInstanceError> {
        self.instances
            .iter_mut()
            .map(|(id, hc)| {
                notify(format!("Stopping instance \"{}\"...", id));
                hc.write()
                    .map(|mut lock| {
                        let _ = lock.stop();
                    })
                    .map_err(|_| {
                        notify(format!("Error stopping instance \"{}\": could not get a lock. Will ignore and proceed shutting down other instances...", id));
                        HolochainInstanceError::InternalFailure(HolochainError::new("Could not get lock on shutdown"))
                    })
            })
            .collect::<Result<Vec<()>, _>>()
            .map(|_| ())
    }

    pub fn instances(&self) -> &InstanceMap {
        &self.instances
    }

    /// Stop and clear all instances
    pub fn shutdown(&mut self) -> Result<(), HolochainInstanceError> {
        self.stop_all_instances()?;
        self.stop_all_interfaces();
        self.signal_multiplexer_kill_switch
            .as_ref()
            .map(|sender| sender.send(()));
        self.instances = HashMap::new();
        Ok(())
    }

    pub fn spawn_network(&mut self) -> Result<SpawnResult, HolochainError> {
        let network_config = self.config.clone().network.ok_or_else(|| {
            HolochainError::ErrorGeneric("attempt to spawn network when not configured".to_string())
        })?;

        match network_config {
            NetworkConfig::N3h(config) => {
                println!(
                    "Spawning network with working directory: {}",
                    config.n3h_persistence_path
                );
                let spawn_result = ipc_spawn(
                    config.n3h_persistence_path.clone(),
                    P2pConfig::load_end_user_config(config.networking_config_file).to_string(),
                    hashmap! {
                        String::from("N3H_MODE") => config.n3h_mode.clone(),
                        String::from("N3H_WORK_DIR") => config.n3h_persistence_path.clone(),
                        String::from("N3H_IPC_SOCKET") => String::from("tcp://127.0.0.1:*"),
                        String::from("N3H_LOG_LEVEL") => config.n3h_log_level.clone(),
                    },
                    2000,
                    true,
                )
                .map_err(|error| {
                    println!("Error while spawning network process: {:?}", error);
                    HolochainError::ErrorGeneric(error.to_string())
                })?;
                println!(
                    "Network spawned with bindings:\n\t - ipc: {}\n\t - p2p: {:?}",
                    spawn_result.ipc_binding, spawn_result.p2p_bindings
                );
                Ok(spawn_result)
            }
            NetworkConfig::Memory(_) => unimplemented!(),
            NetworkConfig::Sim1h(_) => unimplemented!(),
            NetworkConfig::Sim2h(_) => unimplemented!(),
            NetworkConfig::Lib3h(_) => Err(HolochainError::ErrorGeneric(
                "Lib3h Network not implemented".to_string(),
            )),
        }
    }

    fn get_p2p_config(&self) -> P2pConfig {
        self.p2p_config.clone().map(|p2p_config| {

          // TODO replace this hack with a discovery service trait
          let urls : Vec<url::Url> = self.instances.values().map(|instance| {
                    instance
                        .read()
                        .unwrap()
                        .context()
                        .unwrap()
                        .network_state()
                        .unwrap()
                        .network
                        .as_ref()
                        .expect("Network not initialized")
                        .p2p_endpoint()
                }).collect();
            match p2p_config.to_owned().backend_config {
                BackendConfig::Memory(mut config) => {
                    config.bootstrap_nodes =
                        if config.bootstrap_nodes.is_empty() && !urls.is_empty()
                        { vec![urls[0].clone().into()] }
                        else
                        { config.bootstrap_nodes.clone() };
                    let mut p2p_config = p2p_config.clone();
                    p2p_config.backend_config = BackendConfig::Memory(config);
                    p2p_config
                },
                _ => p2p_config.clone()
            }
        }).unwrap_or_else(|| {
            // This should never happen, but we'll throw out an in-memory server config rather than crashing,
            // just to be nice (TODO make proper logging statement)
            println!("warn: instance_network_config called before p2p_config initialized! Using default in-memory network name.");
            P2pConfig::new_with_unique_memory_backend()
        })
    }

    fn initialize_p2p_config(&mut self) -> P2pConfig {
        // if there's no NetworkConfig we won't spawn a network process
        // and instead configure instances to use a unique in-memory network
        if self.config.network.is_none() {
            return P2pConfig::new_with_unique_memory_backend();
        }
        // if there is a config then either we need to spawn a process and get
        // the ipc_uri for it and save it for future calls to `load_config` or
        // we use a (non-empty) uri value that was created from previous calls!
        match self.config.network.clone().unwrap() {
            NetworkConfig::N3h(config) => {
                let uri = config
                    .n3h_ipc_uri
                    .clone()
                    .and_then(|v| if v == "" { None } else { Some(v) })
                    .or_else(|| {
                        self.network_spawn = self.spawn_network().ok();
                        self.network_spawn
                            .as_ref()
                            .map(|spawn| spawn.ipc_binding.clone())
                    });
                let config = P2pConfig::new_ipc_uri(
                    uri,
                    &config.bootstrap_nodes,
                    config.networking_config_file,
                );
                // create an empty network with this config just so the n3h process doesn't
                // kill itself in the case that all instances are closed down (as happens in app-spec)
                let network = P2pNetwork::new(
                    NetHandler::new(Box::new(|_r| Ok(()))),
                    config.clone(),
                    None,
                    None,
                )
                .expect("unable to create conductor keepalive P2pNetwork");
                self.n3h_keepalive_network = Some(network);
                config
            }
            NetworkConfig::Memory(config) => P2pConfig {
                backend_kind: P2pBackendKind::GhostEngineMemory,
                backend_config: BackendConfig::Memory(config),
                maybe_end_user_config: None,
            },
            NetworkConfig::Lib3h(config) => P2pConfig {
                backend_kind: P2pBackendKind::LIB3H,
                backend_config: BackendConfig::Lib3h(config),
                maybe_end_user_config: None,
            },
            NetworkConfig::Sim1h(config) => P2pConfig {
                backend_kind: P2pBackendKind::SIM1H,
                backend_config: BackendConfig::Sim1h(config),
                maybe_end_user_config: None,
            },
            NetworkConfig::Sim2h(config) => P2pConfig {
                backend_kind: P2pBackendKind::SIM2H,
                backend_config: BackendConfig::Sim2h(config),
                maybe_end_user_config: None,
            },
        }
    }

    /// Tries to create all instances configured in the given Configuration object.
    /// Calls `Configuration::check_consistency()` first and clears `self.instances`.
    /// The first time we call this, we also initialize the conductor-wide config
    /// for use with all instances
    pub fn boot_from_config(&mut self) -> Result<(), String> {
        notify("conductor: boot_from_config".into());
        let _ = self.config.check_consistency(&mut self.dna_loader)?;

        if self.p2p_config.is_none() {
            self.p2p_config = Some(self.initialize_p2p_config());
        }

        self.shutdown().map_err(|e| e.to_string())?;

        self.start_signal_multiplexer();
        self.dpki_bootstrap()?;

        for id in self.config.instance_ids_sorted_by_bridge_dependencies()? {
            // We only try to instantiate the instance if it is not running already,
            // which will be the case at least for the DPKI instance which got started
            // specifically in `self.dpki_bootstrap()` above.
            if !self.instances.contains_key(&id) {
                let instance = self.instantiate_from_config(&id).map_err(|error| {
                    format!(
                        "Error while trying to create instance \"{}\": {}",
                        id, error
                    )
                })?;

                self.instances
                    .insert(id.clone(), Arc::new(RwLock::new(instance)));
            }
        }

        for ui_interface_config in self.config.ui_interfaces.clone() {
            notify(format!("adding ui interface {}", &ui_interface_config.id));
            let bundle_config = self
                .config
                .ui_bundle_by_id(&ui_interface_config.bundle)
                .ok_or_else(|| {
                    format!(
                        "UI interface {} references bundle with id {} but no such bundle found",
                        &ui_interface_config.id, &ui_interface_config.bundle
                    )
                })?;
            let connected_dna_interface = ui_interface_config
                .clone()
                .dna_interface
                .map(|interface_id| self.config.interface_by_id(&interface_id).unwrap());

            self.static_servers.insert(
                ui_interface_config.id.clone(),
                StaticServer::from_configs(
                    ui_interface_config,
                    bundle_config,
                    connected_dna_interface,
                ),
            );
        }

        Ok(())
    }

    /// Creates one specific Holochain instance from a given Configuration,
    /// id string and DnaLoader.
    pub fn instantiate_from_config(&mut self, id: &String) -> Result<Holochain, String> {
        self.config.check_consistency(&mut self.dna_loader)?;

        self.config
            .instance_by_id(&id)
            .ok_or_else(|| String::from("Instance not found in config"))
            .and_then(|instance_config| {
                // Build context:
                let mut context_builder = ContextBuilder::new();

                // Agent:
                let agent_id = &instance_config.agent;
                let agent_config = self.config.agent_by_id(agent_id).unwrap();
                let agent_address = self.agent_config_to_id(&agent_config)?;
                if agent_config.test_agent.unwrap_or_default() {
                    // Modify the config so that the public_address is correct.
                    // (The public_address is simply ignored for test_agents, as
                    // it is generated from the agent's name instead of read from
                    // a physical keyfile)
                    self.config.update_agent_address_by_id(agent_id, &agent_address);
                    self.save_config()?;
                }

                context_builder = context_builder.with_agent(agent_address.clone());

                context_builder = context_builder.with_p2p_config(self.get_p2p_config());

                // Signal config:
                let (sender, receiver) = unbounded();
                self.instance_signal_receivers
                    .write()
                    .unwrap()
                    .insert(instance_config.id.clone(), receiver);
                context_builder = context_builder.with_signals(sender);

                // Storage:
                match instance_config.storage {
                    StorageConfiguration::File { path } => {
                        context_builder =
                            context_builder.with_file_storage(path).map_err(|hc_err| {
                                format!("Error creating context: {}", hc_err.to_string())
                            })?
                    }
                    StorageConfiguration::Memory => {
                        context_builder = context_builder.with_memory_storage()
                    }
                    StorageConfiguration::Pickle { path } => {
                        context_builder =
                            context_builder
                                .with_pickle_storage(path)
                                .map_err(|hc_err| {
                                    format!("Error creating context: {}", hc_err.to_string())
                                })?
                    }
                }

                let instance_name = instance_config.id.clone();
                // Conductor API
                let api = self.build_conductor_api(instance_config.id)?;
                context_builder = context_builder.with_conductor_api(api);

                if self.config.logger.state_dump {
                    context_builder = context_builder.with_state_dump_logging();
                }

                // Spawn context
                let context = context_builder.with_instance_name(&instance_name).spawn();

                // Get DNA

                // self.config.dnas.iter_mut().fing(|dna_config| dna_config.id == instance_config.dna)
                // .map(|dna_config| {

                // })
                let dna_config = self.config.dna_by_id(&instance_config.dna).unwrap();
                let dna_file = PathBuf::from(&dna_config.file);
                let mut dna = Arc::get_mut(&mut self.dna_loader).unwrap()(&dna_file).map_err(|_| {
                    HolochainError::ConfigError(format!(
                        "Could not load DNA file \"{}\"",
                        dna_config.file
                    ))
                })?;


                match dna_config.uuid {
                    Some(uuid) => {
                        dna.uuid = uuid;
                        self.config.update_dna_hash_by_id(&dna_config.id, dna.address().to_string());
                        self.save_config()?;
                    },
                    None => {
                        // This is where we are checking the consistency between DNAs: for now we compare
                        // the hash provided in the TOML Conductor config file with the computed hash of
                        // the loaded dna.
                        // NB: we only do this is if the uuid is not set
                        let dna_hash_from_conductor_config = HashString::from(dna_config.hash);
                        let dna_hash_computed = &dna.address();

                        match Arc::get_mut(&mut self.dna_loader)
                            .expect("Fail to get a mutable reference to 'dna loader'.")(&dna_file) {
                            // If the file is correctly loaded, meaning it exists in the file system,
                            // we can operate on its computed DNA hash
                            Ok(dna) => {
                                let dna_hash_computed_from_file = dna.address();
                                Conductor::check_dna_consistency_from_all_sources(
                                    &context,
                                    &dna_hash_from_conductor_config,
                                    &dna_hash_computed,
                                    &dna_hash_computed_from_file, &dna_file)?;
                            },
                            Err(_) => {
                                let msg = format!("Conductor: Could not load DNA file {:?}.", &dna_file);
                                log_error!(context, "{}", msg);

                                // If something is wrong with the DNA file, we only
                                // check the 2 primary sources of DNA's hashes
                                match Conductor::check_dna_consistency(
                                    &dna_hash_from_conductor_config,
                                    &dna_hash_computed) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        let msg = format!("\
                                        Conductor: DNA hashes mismatch: 'Conductor config' != 'Conductor instance': \
                                        '{}' != '{}'",
                                        &dna_hash_from_conductor_config,
                                        &dna_hash_computed);
                                        log_error!(context, "{}", msg);

                                        return Err(e.to_string());
                                    }
                                }
                            }
                        }
                    }
                };

                let context = Arc::new(context);
                               Holochain::load(context.clone())
                    .and_then(|hc| {
                       notify(format!(
                            "Successfully loaded instance {} from storage",
                            id.clone()
                        ));
                        Ok(hc)
                    })
                    .or_else(|loading_error| {
                        // NoneError just means it didn't find a pre-existing state
                        // that's not a problem and so isn't logged as such
                        if loading_error == HolochainError::from(NoneError) {
                           notify("No chain found in the store".to_string());
                        } else {
                            notify(format!(
                                "Failed to load instance {} from storage: {:?}",
                                id.clone(),
                                loading_error
                            ));
                        }
                        notify("Initializing new chain...".to_string());
                        Holochain::new(dna, context)
                        .map_err(|hc_err| hc_err.to_string())
                    })
            })
    }

    pub fn build_conductor_api(
        &mut self,
        instance_id: String,
    ) -> Result<IoHandler, HolochainError> {
        notify(format!(
            "conductor: build_conductor_api instance_id={}, config={:?}",
            instance_id, self.config
        ));
        let instance_config = self.config.instance_by_id(&instance_id)?;
        let agent_id = instance_config.agent.clone();
        let agent_config = self.config.agent_by_id(&agent_id)?;
        let mut api_builder = ConductorApiBuilder::new();
        // Signing callback:
        if let Some(true) = agent_config.holo_remote_key {
            // !!!!!!!!!!!!!!!!!!!!!!!
            // Holo closed-alpha hack:
            // !!!!!!!!!!!!!!!!!!!!!!!
            api_builder = api_builder.with_outsource_signing_callback(
                self.agent_config_to_id(&agent_config)?,
                self.config
                    .signing_service_uri
                    .clone()
                    .expect("holo_remote_key needs signing_service_uri set"),
            );
            api_builder = api_builder.with_outsource_signing_callback(
                self.agent_config_to_id(&agent_config)?,
                self.config
                    .encryption_service_uri
                    .clone()
                    .expect("holo_remote_key needs encryption_service_uri set"),
            );
            api_builder = api_builder.with_outsource_signing_callback(
                self.agent_config_to_id(&agent_config)?,
                self.config
                    .decryption_service_uri
                    .clone()
                    .expect("holo_remote_key needs decryption_service_uri set"),
            );
        } else {
            api_builder = api_builder.with_agent_signature_callback(
                self.get_keybundle_for_agent(&instance_config.agent)?,
            );

            api_builder = api_builder.with_agent_encryption_callback(
                self.get_keybundle_for_agent(&instance_config.agent)?,
            );
            api_builder = api_builder.with_agent_decryption_callback(
                self.get_keybundle_for_agent(&instance_config.agent)?,
            );
            let keystore = self.get_keystore_for_agent(&instance_config.agent)?;
            api_builder = api_builder.with_agent_keystore_functions(keystore);
        }

        // Bridges:
        let id = instance_config.id.clone();
        for bridge in self.config.bridge_dependencies(id.clone()) {
            assert_eq!(bridge.caller_id, id.clone());
            let callee_config = self
                .config
                .instance_by_id(&bridge.callee_id)
                .expect("config.check_consistency()? jumps out if config is broken");
            let callee_instance = self.instances.get(&bridge.callee_id).expect(
                r#"
                    We have to create instances ordered by bridge dependencies such that we
                    can expect the callee to be present here because we need it to create
                    the bridge API"#,
            );

            api_builder =
                api_builder.with_named_instance(bridge.handle.clone(), callee_instance.clone());
            api_builder =
                api_builder.with_named_instance_config(bridge.handle.clone(), callee_config);
        }

        Ok(api_builder.spawn())
    }

    pub fn agent_config_to_id(
        &mut self,
        agent_config: &AgentConfiguration,
    ) -> Result<AgentId, HolochainError> {
        Ok(if let Some(true) = agent_config.holo_remote_key {
            // !!!!!!!!!!!!!!!!!!!!!!!
            // Holo closed-alpha hack:
            // !!!!!!!!!!!!!!!!!!!!!!!
            AgentId::new(&agent_config.name, agent_config.public_address.clone())
        } else {
            let keybundle_arc = self.get_keybundle_for_agent(&agent_config.id)?;
            let keybundle = keybundle_arc.lock().unwrap();
            AgentId::new(&agent_config.name, keybundle.get_id())
        })
    }

    /// Checks if the key for the given agent can be loaded or was already loaded.
    /// Will trigger loading if key is not loaded yet.
    /// Meant to be used in conductor executable to first try to load all keys (which will trigger
    /// passphrase prompts) before bootstrapping the whole config and have prompts appear
    /// in between other initialization output.
    pub fn check_load_key_for_agent(&mut self, agent_id: &String) -> Result<(), String> {
        if let Some(true) = self
            .config
            .agent_by_id(agent_id)
            .and_then(|a| a.holo_remote_key)
        {
            // !!!!!!!!!!!!!!!!!!!!!!!
            // Holo closed-alpha hack:
            // !!!!!!!!!!!!!!!!!!!!!!!
            return Ok(());
        }
        self.get_keystore_for_agent(agent_id)?;
        Ok(())
    }

    /// Checks DNA's hashes from all sources:
    /// - dna_hash_from_conductor_config: from the Conductor configuration
    /// - dna_hash_computed: from the hash computed based on the loaded DNA
    /// and
    /// - dna_hash_computed_from_file: from the hash computed from the loaded DNA of the file.dna
    fn check_dna_consistency_from_all_sources(
        ctx: &holochain_core::context::Context,
        dna_hash_from_conductor_config: &HashString,
        dna_hash_computed: &HashString,
        dna_hash_computed_from_file: &HashString,
        dna_file: &PathBuf,
    ) -> Result<(), HolochainError> {
        match Conductor::check_dna_consistency(&dna_hash_from_conductor_config, &dna_hash_computed)
        {
            Ok(_) => (),
            Err(e) => {
                let msg = format!("\
                                err/Conductor: DNA hashes mismatch: 'Conductor config' != 'Conductor instance': \
                                '{}' != '{}'",
                                &dna_hash_from_conductor_config,
                                &dna_hash_computed);

                log_debug!(ctx, "{}", msg);

                return Err(e);
            }
        }

        match Conductor::check_dna_consistency(
            &dna_hash_from_conductor_config,
            &dna_hash_computed_from_file,
        ) {
            Ok(_) => (),
            Err(e) => {
                let msg = format!("\
                                err/Conductor: DNA hashes mismatch: 'Conductor config' != 'Hash computed from the file {:?}': \
                                '{}' != '{}'",
                                &dna_file,
                                &dna_hash_from_conductor_config,
                                &dna_hash_computed_from_file);

                log_debug!(ctx, "{}", msg);

                return Err(e);
            }
        }

        match Conductor::check_dna_consistency(&dna_hash_computed, &dna_hash_computed_from_file) {
            Ok(_) => (),
            Err(e) => {
                let msg = format!("\
                                err/Conductor: DNA hashes mismatch: 'Conductor instance' != 'Hash computed from the file {:?}': \
                                '{}' != '{}'",
                                &dna_file,
                                &dna_hash_computed,
                                &dna_hash_computed_from_file);
                log_debug!(ctx, "{}", msg);

                return Err(e);
            }
        }
        Ok(())
    }

    /// This is where we check for DNA's hashes consistency.
    /// Only a simple equality check between DNA hashes is currently performed.
    fn check_dna_consistency(
        dna_hash_a: &HashString,
        dna_hash_b: &HashString,
    ) -> Result<(), HolochainError> {
        if *dna_hash_a == *dna_hash_b {
            Ok(())
        } else {
            Err(HolochainError::DnaHashMismatch(
                dna_hash_a.clone(),
                dna_hash_b.clone(),
            ))
        }
    }

    /// Get reference to keystore for given agent ID.
    /// If the key was not loaded (into secure memory) yet, this will use the KeyLoader
    /// to do so.
    pub fn get_keystore_for_agent(
        &mut self,
        agent_id: &String,
    ) -> Result<Arc<Mutex<Keystore>>, String> {
        if !self.agent_keys.contains_key(agent_id) {
            let agent_config = self
                .config
                .agent_by_id(agent_id)
                .ok_or_else(|| format!("Agent '{}' not found", agent_id))?;
            if let Some(true) = agent_config.holo_remote_key {
                return Err("agent is holo_remote, no keystore".to_string());
            }

            let mut keystore = match agent_config.test_agent {
                Some(true) => test_keystore(&agent_config.name),
                _ => {
                    let keystore_file_path = PathBuf::from(agent_config.keystore_file.clone());
                    let keystore = Arc::get_mut(&mut self.key_loader).unwrap()(
                        &keystore_file_path,
                        self.passphrase_manager.clone()
                    )
                    .map_err(|_| {
                        HolochainError::ConfigError(format!(
                            "Could not load keystore \"{}\"",
                            agent_config.keystore_file,
                        ))
                    })?;
                    keystore
                }
            };

            let keybundle = keystore
                .get_keybundle(PRIMARY_KEYBUNDLE_ID)
                .map_err(|err| format!("{}", err,))?;

            if let Some(true) = agent_config.test_agent {
                // don't worry about public_address if this is a test_agent
            } else {
                if agent_config.public_address != keybundle.get_id() {
                    return Err(format!(
                        "Key from file '{}' ('{}') does not match public address {} mentioned in config!",
                        agent_config.keystore_file,
                        keybundle.get_id(),
                        agent_config.public_address,
                    ));
                }
            }

            self.agent_keys
                .insert(agent_id.clone(), Arc::new(Mutex::new(keystore)));
        }
        let keystore_ref = self.agent_keys.get(agent_id).unwrap();
        Ok(keystore_ref.clone())
    }

    /// Get reference to the keybundle stored in the keystore for given agent ID.
    /// If the key was not loaded (into secure memory) yet, this will use the KeyLoader
    /// to do so.
    pub fn get_keybundle_for_agent(
        &mut self,
        agent_id: &String,
    ) -> Result<Arc<Mutex<KeyBundle>>, String> {
        let keystore = self.get_keystore_for_agent(agent_id)?;
        let mut keystore = keystore.lock().unwrap();
        let keybundle = keystore
            .get_keybundle(PRIMARY_KEYBUNDLE_ID)
            .map_err(|err| format!("{}", err))?;
        Ok(Arc::new(Mutex::new(keybundle)))
    }

    fn start_interface(&mut self, config: &InterfaceConfiguration) -> Result<(), String> {
        if self.interface_threads.contains_key(&config.id) {
            return Err(format!("Interface {} already started!", config.id));
        }
        notify(format!("Starting interface '{}'.", config.id));
        let handle = self.spawn_interface_thread(config.clone());
        self.interface_threads.insert(config.id.clone(), handle);
        Ok(())
    }

    /// Default DnaLoader that actually reads files from the filesystem
    pub fn load_dna(file: &PathBuf) -> HcResult<Dna> {
        notify(format!("Reading DNA from {}", file.display()));
        let mut f = File::open(file)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        Dna::try_from(JsonString::from_json(&contents)).map_err(|err| err.into())
    }

    /// Default KeyLoader that actually reads files from the filesystem
    fn load_key(
        file: &PathBuf,
        passphrase_manager: Arc<PassphraseManager>
    ) -> Result<Keystore, HolochainError> {
        notify(format!("Reading keystore from {}", file.display()));

        let keystore = Keystore::new_from_file(file.clone(), passphrase_manager, None)?;
        Ok(keystore)
    }

    fn copy_ui_dir(source: &PathBuf, dest: &PathBuf) -> Result<(), HolochainError> {
        notify(format!(
            "Copying UI from {} to {}",
            source.display(),
            dest.display()
        ));
        fs::create_dir_all(dest).map_err(|_| {
            HolochainError::ErrorGeneric(format!("Could not directory structure {:?}", dest))
        })?;
        fs_extra::dir::copy(&source, &dest, &fs_extra::dir::CopyOptions::new())
            .map_err(|e| HolochainError::ErrorGeneric(e.to_string()))?;
        Ok(())
    }

    fn make_interface_handler(&self, interface_config: &InterfaceConfiguration) -> IoHandler {
        let mut conductor_api_builder = ConductorApiBuilder::new();
        for instance_ref_config in interface_config.instances.iter() {
            let id = &instance_ref_config.id;
            let name = instance_ref_config.alias.as_ref().unwrap_or(id).clone();

            let instance = self.instances.get(id);
            let instance_config = self.config.instance_by_id(id);
            if instance.is_none() || instance_config.is_none() {
                continue;
            }

            let instance = instance.unwrap();
            let instance_config = instance_config.unwrap();

            conductor_api_builder = conductor_api_builder
                .with_named_instance(name.clone(), instance.clone())
                .with_named_instance_config(name.clone(), instance_config)
        }

        if interface_config.admin {
            conductor_api_builder = conductor_api_builder
                .with_admin_dna_functions()
                .with_admin_ui_functions()
                .with_test_admin_functions()
                .with_debug_functions();
        }

        conductor_api_builder.spawn()
    }

    fn spawn_interface_thread(&self, interface_config: InterfaceConfiguration) -> Sender<()> {
        let dispatcher = self.make_interface_handler(&interface_config);
        // The "kill switch" is the channel which allows the interface to be stopped from outside its thread
        let (kill_switch_tx, kill_switch_rx) = unbounded();

        let iface = make_interface(&interface_config);
        let (broadcaster, _handle) = iface
            .run(dispatcher, kill_switch_rx)
            .map_err(|error| {
                error!(
                    "conductor: Error running interface '{}': {}",
                    interface_config.id, error
                );
                error
            })
            .unwrap();
        debug!("conductor: adding broadcaster to map {:?}", broadcaster);

        {
            self.interface_broadcasters
                .write()
                .unwrap()
                .insert(interface_config.id.clone(), broadcaster);
        }

        kill_switch_tx
    }

    pub fn dna_dir_path(&self) -> PathBuf {
        self.config.persistence_dir.join("dna")
    }

    pub fn config_path(&self) -> PathBuf {
        self.config.persistence_dir.join("conductor-config.toml")
    }

    pub fn instance_storage_dir_path(&self) -> PathBuf {
        self.config.persistence_dir.join("storage")
    }

    pub fn save_config(&self) -> Result<(), HolochainError> {
        fs::create_dir_all(&self.config.persistence_dir).map_err(|_| {
            HolochainError::ErrorGeneric(format!(
                "Could not directory structure {:?}",
                self.config.persistence_dir
            ))
        })?;
        let mut file = File::create(&self.config_path()).map_err(|_| {
            HolochainError::ErrorGeneric(format!(
                "Could not create file at {:?}",
                self.config_path()
            ))
        })?;

        file.write(serialize_configuration(&self.config)?.as_bytes())
            .map_err(|_| {
                HolochainError::ErrorGeneric(format!(
                    "Could not save config to {:?}",
                    self.config_path()
                ))
            })?;
        Ok(())
    }

    pub fn save_dna(&self, dna: &Dna) -> Result<PathBuf, HolochainError> {
        let file_path = self
            .dna_dir_path()
            .join(dna.address().to_string())
            .with_extension(DNA_EXTENSION);
        fs::create_dir_all(&self.dna_dir_path())?;
        self.save_dna_to(dna, file_path)
    }

    pub fn save_dna_to(&self, dna: &Dna, path: PathBuf) -> Result<PathBuf, HolochainError> {
        let file = File::create(&path).map_err(|e| {
            HolochainError::ConfigError(format!(
                "Error writing DNA to {}, {}",
                path.to_str().unwrap().to_string(),
                e.to_string()
            ))
        })?;
        serde_json::to_writer_pretty(&file, dna)?;
        Ok(path)
    }

    /// check for determining if the conductor is using dpki to manage instance keys
    pub fn using_dpki(&self) -> bool {
        self.config.dpki.is_some()
    }

    /// returns the instance_id of the dpki app if it is configured
    pub fn dpki_instance_id(&self) -> Option<String> {
        match self.config.dpki {
            Some(ref dpki) => Some(dpki.instance_id.clone()),
            None => None,
        }
    }

    /// returns the init_params for the dpki app if it is configured
    pub fn dpki_init_params(&self) -> Option<String> {
        match self.config.dpki {
            Some(ref dpki) => Some(dpki.init_params.clone()),
            None => None,
        }
    }

    /// bootstraps the dpki app if configured
    pub fn dpki_bootstrap(&mut self) -> Result<(), HolochainError> {
        // Checking if there is a dpki instance
        if self.using_dpki() {
            notify("DPKI configured. Starting DPKI instance...".to_string());

            self.start_dpki_instance()
                .map_err(|err| format!("Error starting DPKI instance: {:?}", err))?;
            let dpki_instance_id = self
                .dpki_instance_id()
                .expect("We assume there is a DPKI instance since we just started it above..");

            notify(format!(
                "Instance '{}' running as DPKI instance.",
                dpki_instance_id
            ));

            let instance = self.instances.get(&dpki_instance_id)?;
            let hc_lock = instance.clone();
            let hc_lock_inner = hc_lock.clone();
            let mut hc = hc_lock_inner.write().unwrap();

            if !hc.dpki_is_initialized()? {
                notify("DPKI is not initialized yet (i.e. running for the first time). Calling 'init'...".to_string());
                hc.dpki_init(self.dpki_init_params().unwrap())?;
                notify("DPKI initialization done!".to_string());
            }
        }
        Ok(())
    }
}

/// This can eventually be dependency injected for third party Interface definitions
fn make_interface(interface_config: &InterfaceConfiguration) -> Box<dyn Interface> {
    use interface_impls::{http::HttpInterface, websocket::WebsocketInterface};
    match interface_config.driver {
        InterfaceDriver::Websocket { port } => Box::new(WebsocketInterface::new(port)),
        InterfaceDriver::Http { port } => Box::new(HttpInterface::new(port)),
        _ => unimplemented!(),
    }
}

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use conductor::{passphrase_manager::PassphraseManager, test_admin::ConductorTestAdmin};
    use key_loaders::mock_passphrase_manager;
    use keystore::{test_hash_config, Keystore, Secret, PRIMARY_KEYBUNDLE_ID};
    extern crate tempfile;
    use crate::config::load_configuration;
    use holochain_core::{
        action::Action, nucleus::actions::call_zome_function::make_cap_request_for_call,
        signal::signal_channel,
    };
    use holochain_core_types::dna;
    use holochain_dpki::{
        key_bundle::KeyBundle, CRYPTO, SEED_SIZE,
    };
    use holochain_persistence_api::cas::content::Address;
    use holochain_wasm_utils::wasm_target_dir;
    use lib3h_sodium::secbuf::SecBuf;
    use std::{
        fs::{File, OpenOptions},
        io::Write,
        path::PathBuf,
    };

    use self::tempfile::tempdir;
    use holochain_core_types::dna::{
        bridges::{Bridge, BridgeReference},
        fn_declarations::{FnDeclaration, Trait, TraitFns},
    };
    use std::collections::BTreeMap;
    use test_utils::*;

    //    commented while test_signals_through_admin_websocket is broken
    //    extern crate ws;
    //    use self::ws::{connect, Message};
    //    extern crate parking_lot;

    pub fn test_dna_loader() -> DnaLoader {
        let loader = Box::new(|path: &PathBuf| {
            Ok(match path.to_str().unwrap().as_ref() {
                "bridge/callee.dna" => callee_dna(),
                "bridge/caller.dna" => caller_dna(),
                "bridge/caller_dna_ref.dna" => caller_dna_with_dna_reference(),
                "bridge/caller_bogus_trait_ref.dna" => caller_dna_with_bogus_trait_reference(),
                "bridge/caller_without_required.dna" => caller_dna_without_required(),
                _ => Dna::try_from(JsonString::from_json(&example_dna_string())).unwrap(),
            })
        })
            as Box<dyn FnMut(&PathBuf) -> Result<Dna, HolochainError> + Send + Sync>;
        Arc::new(loader)
    }

    pub fn test_key_loader() -> KeyLoader {
        let loader = Box::new(
            |path: &PathBuf, _pm: Arc<PassphraseManager>| {
                match path.to_str().unwrap().as_ref() {
                    "holo_tester1.key" => Ok(test_keystore(1)),
                    "holo_tester2.key" => Ok(test_keystore(2)),
                    "holo_tester3.key" => Ok(test_keystore(3)),
                    unknown => Err(HolochainError::ErrorGeneric(format!(
                        "No test keystore for {}",
                        unknown
                    ))),
                }
            },
        )
            as Box<
                dyn FnMut(
                        &PathBuf,
                        Arc<PassphraseManager>,
                    ) -> Result<Keystore, HolochainError>
                    + Send
                    + Sync,
            >;
        Arc::new(loader)
    }

    pub fn test_keystore(index: u8) -> Keystore {
        let agent_name = format!("test-agent-{}", index);
        let mut keystore = Keystore::new(
            mock_passphrase_manager(agent_name.clone()),
            test_hash_config(),
        )
        .unwrap();

        // Create deterministic seed
        let mut seed = CRYPTO.buf_new_insecure(SEED_SIZE);
        let mock_seed: Vec<u8> = (1..SEED_SIZE).map(|e| e as u8 + index).collect();
        seed.write(0, mock_seed.as_slice())
            .expect("SecBuf must be writeable");
        let mut secret_secbuf = SecBuf::with_secure(seed.len());
        secret_secbuf
            .from_array(&seed)
            .expect("Could not create from array");
        let secret = Arc::new(Mutex::new(Secret::Seed(secret_secbuf)));
        keystore.add("root_seed", secret).unwrap();

        keystore
            .add_keybundle_from_seed("root_seed", PRIMARY_KEYBUNDLE_ID)
            .unwrap();
        keystore
    }

    pub fn test_keybundle(index: u8) -> KeyBundle {
        let mut keystore = test_keystore(index);
        keystore.get_keybundle(PRIMARY_KEYBUNDLE_ID).unwrap()
    }

    pub fn test_toml(websocket_port: u16, http_port: u16) -> String {
        format!(
            r#"
    [[agents]]
    id = "test-agent-1"
    name = "Holo Tester 1"
    public_address = "{tkb1}"
    keystore_file = "holo_tester1.key"

    [[agents]]
    id = "test-agent-2"
    name = "Holo Tester 2"
    public_address = "{tkb2}"
    keystore_file = "holo_tester2.key"

    [[agents]]
    id = "test-agent-3"
    name = "Holo Tester 3"
    public_address = "{tkb3}"
    keystore_file = "holo_tester3.key"

    [[dnas]]
    id = "test-dna"
    file = "app_spec.dna.json"
    hash = "QmaJiTs75zU7kMFYDkKgrCYaH8WtnYNkmYX3tPt7ycbtRq"

    [[dnas]]
    id = "bridge-callee"
    file = "bridge/callee.dna"
    hash = "{bridge_callee_hash}"

    [[dnas]]
    id = "bridge-caller"
    file = "bridge/caller.dna"
    hash = "{bridge_caller_hash}"

    [[instances]]
    id = "test-instance-1"
    dna = "bridge-callee"
    agent = "test-agent-1"
        [instances.storage]
        type = "memory"

    [[instances]]
    id = "test-instance-2"
    dna = "test-dna"
    agent = "test-agent-2"
        [instances.storage]
        type = "memory"

    [[instances]]
    id = "bridge-caller"
    dna = "bridge-caller"
    agent = "test-agent-3"
        [instances.storage]
        type = "memory"

    [[interfaces]]
    id = "test-interface-1"
    admin = true
        [interfaces.driver]
        type = "websocket"
        port = {ws_port}
        [[interfaces.instances]]
        id = "test-instance-1"
        [[interfaces.instances]]
        id = "test-instance-2"

    [[interfaces]]
    id = "test-interface-2"
    [interfaces.driver]
    type = "http"
    port = {http_port}
        [[interfaces.instances]]
        id = "test-instance-1"
        [[interfaces.instances]]
        id = "test-instance-2"

    [[bridges]]
    caller_id = "bridge-caller"
    callee_id = "test-instance-1"
    handle = "DPKI"

    [[bridges]]
    caller_id = "bridge-caller"
    callee_id = "test-instance-2"
    handle = "happ-store"

    [[bridges]]
    caller_id = "bridge-caller"
    callee_id = "test-instance-1"
    handle = "test-callee"
    "#,
            tkb1 = test_keybundle(1).get_id(),
            tkb2 = test_keybundle(2).get_id(),
            tkb3 = test_keybundle(3).get_id(),
            ws_port = websocket_port,
            http_port = http_port,
            bridge_callee_hash = callee_dna().address(),
            bridge_caller_hash = caller_dna().address(),
        )
    }

    pub fn test_conductor(websocket_port: u16, http_port: u16) -> Conductor {
        let config =
            load_configuration::<Configuration>(&test_toml(websocket_port, http_port)).unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        conductor.boot_from_config().unwrap();
        conductor
    }

    fn test_conductor_with_signals(signal_tx: SignalSender) -> Conductor {
        let config = load_configuration::<Configuration>(&test_toml(8888, 8889)).unwrap();
        let mut conductor = Conductor::from_config(config.clone()).with_signal_channel(signal_tx);
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        conductor.boot_from_config().unwrap();
        conductor
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
                        "config": {},
                        "entry_types": {
                            "": {
                                "description": "",
                                "sharing": "public"
                            }
                        },
                        "traits": {
                            "test": {
                                "functions": ["test"]
                             }
                        },
                        "fn_declarations": [
                            {
                                "name": "test",
                                "inputs": [
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
                        ],
                        "code": {
                            "code": "AAECAw=="
                        },
                        "bridges": [
                            {
                                "presence": "optional",
                                "handle": "my favourite instance!",
                                "reference": {
                                    "traits": {}
                                }
                            }
                        ]
                    }
                }
            }"#
        .to_string()
    }

    #[test]
    fn test_default_dna_loader() {
        let tempdir = tempdir().unwrap();
        let file_path = tempdir.path().join("test.dna.json");
        let mut tmp_file = File::create(file_path.clone()).unwrap();
        writeln!(tmp_file, "{}", example_dna_string()).unwrap();
        match Conductor::load_dna(&file_path) {
            Ok(dna) => {
                assert_eq!(dna.name, "my dna");
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_conductor_boot_from_config() {
        let mut conductor = test_conductor(10001, 10002);
        assert_eq!(conductor.instances.len(), 3);

        conductor.start_all_instances().unwrap();
        conductor.start_all_interfaces();
        conductor.stop_all_instances().unwrap();
    }

    #[test]
    /// Here we test if we correctly check for consistency in DNA hashes: possible sources are:
    /// - DNA hash from Conductor configuration
    /// - computed DNA hash from loaded instance
    fn test_check_dna_consistency() {
        let toml = test_toml(10041, 10042);

        let config = load_configuration::<Configuration>(&toml).unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        assert_eq!(
            conductor.boot_from_config(),
            Ok(()),
            "Conductor failed to boot from config"
        );

        // Tests equality
        let a = HashString::from("QmYRM4rh8zmSLaxyShYtv9PBDdQkXuyPieJTZ1e5GZqeeh");
        let b = HashString::from("QmYRM4rh8zmSLaxyShYtv9PBDdQkXuyPieJTZ1e5GZqeeh");
        assert_eq!(
            Conductor::check_dna_consistency(&a, &b),
            Ok(()),
            "DNA consistency check Fail."
        );

        // Tests INequality
        let b = HashString::from("QmQVLgFxUpd1ExVkBzvwASshpG6fmaJGxDEgf1cFf7S73a");
        assert_ne!(
            Conductor::check_dna_consistency(&a, &b),
            Ok(()),
            "DNA consistency check Fail."
        );
    }

    #[test]
    /// This is supposed to fail to show if we are properly bailing when there is
    /// a decrepency btween DNA hashes.
    fn test_check_dna_consistency_err() {
        let a = HashString::from("QmYRM4rh8zmSLaxyShYtv9PBDdQkXuyPieJTZ1e5GZqeeh");
        let b = HashString::from("QmZAQkpkXhfRcSgBJX4NYyqWCyMnkvuF7X2RkPgqihGMrR");

        assert_eq!(
            Conductor::check_dna_consistency(&a, &b),
            Err(HolochainError::DnaHashMismatch(a, b)),
            "DNA consistency check Fail."
        );

        let a = HashString::from("QmYRM4rh8zmSLaxyShYtv9PBDdQkXuyPieJTZ1e5GZqeeh");
        let b = HashString::from(String::default());

        assert_eq!(
            Conductor::check_dna_consistency(&a, &b),
            Err(HolochainError::DnaHashMismatch(a, b)),
            "DNA consistency check Fail."
        )
    }

    #[test]
    fn test_serialize_and_load_with_test_agents() {
        let mut conductor = test_conductor(10091, 10092);

        conductor
            .add_test_agent("test-agent-id".into(), "test-agent-name".into())
            .expect("could not add test agent");

        let config_toml_string =
            serialize_configuration(&conductor.config()).expect("Could not serialize config");
        let serialized_config = load_configuration::<Configuration>(&config_toml_string)
            .expect("Could not deserialize toml");

        let mut reanimated_conductor = Conductor::from_config(serialized_config);
        reanimated_conductor.dna_loader = test_dna_loader();
        reanimated_conductor.key_loader = test_key_loader();

        assert_eq!(
            reanimated_conductor
                .config()
                .agents
                .iter()
                .filter_map(|agent| agent.test_agent)
                .count(),
            1
        );
        reanimated_conductor
            .boot_from_config()
            .expect("Could not boot the conductor with test agent")
    }

    #[test]
    fn test_check_dna_consistency_from_dna_file() {
        let fixture = String::from(
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
                        "config": {},
                        "entry_types": {
                            "": {
                                "description": "",
                                "sharing": "public"
                            }
                        },
                        "traits": {
                            "test": {
                                "functions": ["test"]
                             }
                        },
                        "fn_declarations": [
                            {
                                "name": "test",
                                "inputs": [
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
                        ],
                        "code": {
                            "code": "AAECAw=="
                        }
                    }
                }
            }"#,
        );
        let dna_hash_from_file = HashString::from(
            Dna::try_from(JsonString::from_json(&fixture))
                .expect(&format!("Fail to load DNA from raw string: {}", fixture))
                .address(),
        );
        let dna_hash_computed = HashString::from("QmNPCDBhr6BDBBVWG4mBEVFfhyjsScURYdZoV3fDpzjzgb");

        assert_eq!(
            Conductor::check_dna_consistency(&dna_hash_from_file, &dna_hash_computed),
            Ok(()),
            "DNA consistency from DNA file check Fail."
        );
    }

    //#[test]
    // Default config path ~/.holochain/conductor/conductor-config.toml won't work in CI
    fn _test_conductor_save_and_load_config_default_location() {
        let conductor = test_conductor(10011, 10012);
        assert_eq!(conductor.save_config(), Ok(()));

        let mut toml = String::new();

        let mut file = OpenOptions::new()
            .read(true)
            .open(&conductor.config_path())
            .expect("Could not open config file");
        file.read_to_string(&mut toml)
            .expect("Could not read config file");

        let restored_config =
            load_configuration::<Configuration>(&toml).expect("could not load config");
        assert_eq!(
            serialize_configuration(&conductor.config),
            serialize_configuration(&restored_config)
        )
    }

    #[test]
    fn test_conductor_signal_handler() {
        let (signal_tx, signal_rx) = signal_channel();
        let _conductor = test_conductor_with_signals(signal_tx);

        test_utils::expect_action(&signal_rx, |action| match action {
            Action::InitializeChain(_) => true,
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

    pub fn callee_wat() -> String {
        r#"
(module

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "__hdk_validate_app_entry")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )

    (func
        (export "__hdk_validate_agent_entry")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )

    (func
        (export "__hdk_validate_link")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )


    (func
        (export "__hdk_get_validation_package_for_entry_type")
        (param $allocation i64)
        (result i64)

        ;; This writes "Entry" into memory
        (i64.store (i32.const 0) (i64.const 34))
        (i64.store (i32.const 1) (i64.const 69))
        (i64.store (i32.const 2) (i64.const 110))
        (i64.store (i32.const 3) (i64.const 116))
        (i64.store (i32.const 4) (i64.const 114))
        (i64.store (i32.const 5) (i64.const 121))
        (i64.store (i32.const 6) (i64.const 34))

        (i64.const 7)
    )

    (func
        (export "__hdk_get_validation_package_for_link")
        (param $allocation i64)
        (result i64)

        ;; This writes "Entry" into memory
        (i64.store (i32.const 0) (i64.const 34))
        (i64.store (i32.const 1) (i64.const 69))
        (i64.store (i32.const 2) (i64.const 110))
        (i64.store (i32.const 3) (i64.const 116))
        (i64.store (i32.const 4) (i64.const 114))
        (i64.store (i32.const 5) (i64.const 121))
        (i64.store (i32.const 6) (i64.const 34))

        (i64.const 7)
    )

    (func
        (export "__list_traits")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )

    (func
        (export "__list_functions")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )

    (func
        (export "hello")
        (param $allocation i64)
        (result i64)

        ;; This writes "Holo World" into memory
        (i64.store (i32.const 0) (i64.const 72))
        (i64.store (i32.const 1) (i64.const 111))
        (i64.store (i32.const 2) (i64.const 108))
        (i64.store (i32.const 3) (i64.const 111))
        (i64.store (i32.const 4) (i64.const 32))
        (i64.store (i32.const 5) (i64.const 87))
        (i64.store (i32.const 6) (i64.const 111))
        (i64.store (i32.const 7) (i64.const 114))
        (i64.store (i32.const 8) (i64.const 108))
        (i64.store (i32.const 9) (i64.const 100))

        (i64.const 10)
    )
)
                "#
        .to_string()
    }

    fn bridge_call_fn_declaration() -> FnDeclaration {
        FnDeclaration {
            name: String::from("hello"),
            inputs: vec![],
            outputs: vec![dna::fn_declarations::FnParameter {
                name: String::from("greeting"),
                parameter_type: String::from("String"),
            }],
        }
    }

    fn callee_dna() -> Dna {
        let wat = &callee_wat();
        let mut dna = create_test_dna_with_wat("greeter", Some(wat));
        dna.uuid = String::from("basic_bridge_call");
        let fn_declaration = bridge_call_fn_declaration();

        {
            let zome = dna.zomes.get_mut("greeter").unwrap();
            zome.fn_declarations.push(fn_declaration.clone());
            zome.traits
                .get_mut("hc_public")
                .unwrap()
                .functions
                .push(fn_declaration.name.clone());
            zome.traits.insert(
                String::from("greetable"),
                TraitFns {
                    functions: vec![fn_declaration.name.clone()],
                },
            );
        }

        dna
    }

    fn caller_dna() -> Dna {
        let mut path = PathBuf::new();

        path.push(wasm_target_dir(
            &String::from("conductor_api").into(),
            &String::from("test-bridge-caller").into(),
        ));
        let wasm_path_component: PathBuf = [
            String::from("wasm32-unknown-unknown"),
            String::from("release"),
            String::from("test_bridge_caller.wasm"),
        ]
        .iter()
        .collect();
        path.push(wasm_path_component);

        let wasm = create_wasm_from_file(&path);
        let defs = create_test_defs_with_fn_names(vec![
            "call_bridge".to_string(),
            "call_bridge_error".to_string(),
        ]);
        let mut dna = create_test_dna_with_defs("test_zome", defs, &wasm);
        dna.uuid = String::from("basic_bridge_call");
        {
            let zome = dna.zomes.get_mut("test_zome").unwrap();
            zome.bridges.push(Bridge {
                presence: BridgePresence::Required,
                handle: String::from("test-callee"),
                reference: BridgeReference::Trait {
                    traits: btreemap! {
                        String::from("greetable") => Trait{
                            functions: vec![bridge_call_fn_declaration()]
                        }
                    },
                },
            });
            zome.bridges.push(Bridge {
                presence: BridgePresence::Optional,
                handle: String::from("DPKI"),
                reference: BridgeReference::Trait {
                    traits: BTreeMap::new(),
                },
            });
            zome.bridges.push(Bridge {
                presence: BridgePresence::Optional,
                handle: String::from("happ-store"),
                reference: BridgeReference::Trait {
                    traits: BTreeMap::new(),
                },
            });
        }

        dna
    }

    #[test]
    fn basic_bridge_call_roundtrip() {
        let config = load_configuration::<Configuration>(&test_toml(10021, 10022)).unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        conductor
            .boot_from_config()
            .expect("Test config must be sane");
        conductor
            .start_all_instances()
            .expect("Instances must be spawnable");
        let caller_instance = conductor.instances["bridge-caller"].clone();
        let instance = caller_instance.write().unwrap();

        let cap_call = {
            let context = instance.context().unwrap();
            make_cap_request_for_call(
                context.clone(),
                Address::from(context.clone().agent_id.address()),
                "call_bridge",
                JsonString::empty_object(),
            )
        };
        let result = instance
            .call("test_zome", cap_call, "call_bridge", "{}")
            .unwrap();

        // "Holo World" comes for the callee_wat above which runs in the callee instance
        assert_eq!(result, JsonString::from("Holo World"));
    }

    #[test]
    fn basic_bridge_call_error() {
        let config = load_configuration::<Configuration>(&test_toml(10041, 10042)).unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        conductor
            .boot_from_config()
            .expect("Test config must be sane");
        conductor
            .start_all_instances()
            .expect("Instances must be spawnable");
        let caller_instance = conductor.instances["bridge-caller"].clone();
        let instance = caller_instance.write().unwrap();

        let cap_call = {
            let context = instance.context().unwrap();
            make_cap_request_for_call(
                context.clone(),
                Address::from(context.clone().agent_id.address()),
                "call_bridge_error",
                JsonString::empty_object(),
            )
        };
        let result = instance.call("test_zome", cap_call, "call_bridge_error", "{}");

        assert!(result.is_ok());
        assert!(result.unwrap().to_string().contains("Holochain Instance Error: Zome function \'non-existent-function\' not found in Zome \'greeter\'"));
    }

    #[test]
    fn error_if_required_bridge_missing() {
        let mut config = load_configuration::<Configuration>(&test_toml(10061, 10062)).unwrap();
        config.bridges.clear();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();

        let result = conductor.boot_from_config();
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Required bridge \'test-callee\' for instance \'bridge-caller\' missing",
        );
    }

    fn caller_dna_with_dna_reference() -> Dna {
        let mut dna = caller_dna();
        {
            let bridge = dna
                .zomes
                .get_mut("test_zome")
                .unwrap()
                .bridges
                .get_mut(0)
                .unwrap();
            bridge.reference = BridgeReference::Address {
                dna_address: Address::from("fake bridge reference"),
            };
        }
        dna
    }

    fn caller_dna_with_bogus_trait_reference() -> Dna {
        let mut dna = caller_dna();
        {
            let bridge = dna
                .zomes
                .get_mut("test_zome")
                .unwrap()
                .bridges
                .get_mut(0)
                .unwrap();
            let mut fn_declaration = bridge_call_fn_declaration();
            fn_declaration
                .inputs
                .push(dna::fn_declarations::FnParameter {
                    name: String::from("additional_parameter"),
                    parameter_type: String::from("String"),
                });
            bridge.reference = BridgeReference::Trait {
                traits: btreemap! {
                    String::from("greetable") => Trait{
                        functions: vec![fn_declaration]
                    }
                },
            };
        }
        dna
    }

    fn caller_dna_without_required() -> Dna {
        let mut dna = caller_dna();
        {
            let bridge = dna
                .zomes
                .get_mut("test_zome")
                .unwrap()
                .bridges
                .get_mut(0)
                .unwrap();
            bridge.presence = BridgePresence::Optional;
            bridge.reference = BridgeReference::Trait {
                traits: BTreeMap::new(),
            };
        }
        dna
    }

    pub fn bridge_dna_ref_test_toml(caller_dna: &str, callee_dna: &str) -> String {
        format!(
            r#"
    [[agents]]
    id = "test-agent-1"
    name = "Holo Tester 1"
    public_address = "{}"
    keystore_file = "holo_tester1.key"

    [[dnas]]
    id = "bridge-callee"
    file = "{}"
    hash = "Qm328wyq38924y"

    [[dnas]]
    id = "bridge-caller"
    file = "{}"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "bridge-callee"
    dna = "bridge-callee"
    agent = "test-agent-1"
    [instances.storage]
    type = "memory"

    [[instances]]
    id = "bridge-caller"
    dna = "bridge-caller"
    agent = "test-agent-1"
    [instances.storage]
    type = "memory"

    [[bridges]]
    caller_id = "bridge-caller"
    callee_id = "bridge-callee"
    handle = "test-callee"
    "#,
            test_keybundle(1).get_id(),
            callee_dna,
            caller_dna,
        )
    }

    #[test]
    fn error_if_bridge_reference_dna_mismatch() {
        let config = load_configuration::<Configuration>(&bridge_dna_ref_test_toml(
            "bridge/caller_dna_ref.dna",
            "bridge/callee_dna.dna",
        ))
        .unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        let result = conductor.boot_from_config();

        assert!(result.is_err());
        println!("{:?}", result);
        assert!(result.err().unwrap().starts_with(
            "Bridge \'test-callee\' of caller instance \'bridge-caller\' requires callee to be DNA with hash \'fake bridge reference\', but the configured instance \'bridge-callee\' runs DNA with hash"
        ));
    }

    #[test]
    fn error_if_bridge_reference_trait_mismatch() {
        let config = load_configuration::<Configuration>(&bridge_dna_ref_test_toml(
            "bridge/caller_bogus_trait_ref.dna",
            "bridge/callee_dna.dna",
        ))
        .unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        let result = conductor.boot_from_config();

        assert!(result.is_err());
        println!("{:?}", result);
        assert_eq!(
            result.err().unwrap(),
            "Bridge \'test-callee\' of instance \'bridge-caller\' requires callee to to implement trait \'greetable\' with functions: [FnDeclaration { name: \"hello\", inputs: [FnParameter { parameter_type: \"String\", name: \"additional_parameter\" }], outputs: [FnParameter { parameter_type: \"String\", name: \"greeting\" }] }]",
        );
    }

    #[test]
    fn fails_if_key_address_does_not_match() {
        // Config with well formatted public address but differing to the deterministic key
        // created by test_key_loader for "holo_tester1.key"
        let config = load_configuration::<Configuration>(r#"
                [[agents]]
                id = "test-agent-1"
                name = "Holo Tester 1"
                public_address = "HoloTester1-----------------------------------------------------------------------AAACZp4xHB"
                keystore_file = "holo_tester1.key"

                [[dnas]]
                id = "test-dna"
                file = "app_spec.dna.json"
                hash = "QmZAQkpkXhfRcSgBJX4NYyqWCyMnkvuF7X2RkPgqihGMrR"

                [[instances]]
                id = "test-instance-1"
                dna = "test-dna"
                agent = "test-agent-1"
                    [instances.storage]
                    type = "memory"
                "#
        ).unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        assert_eq!(
            conductor.boot_from_config(),
            Err("Error while trying to create instance \"test-instance-1\": Key from file \'holo_tester1.key\' (\'HcSCI7T6wQ5t4nffbjtUk98Dy9fa79Ds6Uzg8nZt8Fyko46ikQvNwfoCfnpuy7z\') does not match public address HoloTester1-----------------------------------------------------------------------AAACZp4xHB mentioned in config!"
                .to_string()),
        );
    }

    #[test]
    // flaky test
    // signal ordering is not deterministic nor is timing
    // test should poll and allow signals in different orders
    // OR
    // test should be totally removed because this is really an integration test
    #[cfg(feature = "broken-tests")]
    fn test_signals_through_admin_websocket() {
        let mut conductor = test_conductor(10031, 10032);
        let _ = conductor.start_all_instances();
        conductor.start_all_interfaces();
        thread::sleep(Duration::from_secs(2));
        // parking_lot::Mutex is an alternative Mutex that does not get poisoned if one of the
        // threads panic. Here it helps getting the causing assertion panic to be printed
        // instead of masking that with a panic of the thread below which makes it hard to see
        // why this test fails, if it fails.
        let signals: Arc<parking_lot::Mutex<Vec<String>>> =
            Arc::new(parking_lot::Mutex::new(Vec::new()));
        let signals_clone = signals.clone();
        let websocket_thread = thread::spawn(|| {
            connect("ws://127.0.0.1:10031", move |_| {
                let s = signals_clone.clone();
                move |msg: Message| {
                    s.lock().push(msg.to_string());
                    Ok(())
                }
            })
            .unwrap();
        });

        let result = {
            let lock = conductor.instances.get("bridge-caller").unwrap();
            let mut bridge_caller = lock.write().unwrap();
            let cap_call = {
                let context = bridge_caller.context();
                make_cap_request_for_call(
                    context.clone(),
                    Address::from(context.clone().agent_id.address()),
                    "call_bridge",
                    JsonString::empty_object(),
                )
            };
            bridge_caller.call(
                "test_zome",
                cap_call,
                "call_bridge",
                &JsonString::empty_object().to_string(),
            )
        };

        assert!(result.is_ok());
        thread::sleep(Duration::from_secs(2));
        conductor.stop_all_interfaces();
        websocket_thread
            .join()
            .expect("Could not join websocket thread");
        let received_signals = signals.lock().clone();

        assert!(received_signals.len() >= 3);
        assert!(received_signals[0]
            .starts_with("{\"signal\":{\"Trace\":\"SignalZomeFunctionCall(ZomeFnCall {"));
        assert!(received_signals[1]
            .starts_with("{\"signal\":{\"Trace\":\"SignalZomeFunctionCall(ZomeFnCall {"));
        assert!(received_signals[2].starts_with(
            "{\"signal\":{\"Trace\":\"ReturnZomeFunctionResult(ExecuteZomeFnResponse {"
        ));
    }

    #[test]
    fn test_start_stop_instance() {
        let mut conductor = test_conductor(10051, 10052);
        assert_eq!(
            conductor.start_instance(&String::from("test-instance-1")),
            Ok(()),
        );
        assert_eq!(
            conductor.start_instance(&String::from("test-instance-1")),
            Err(HolochainInstanceError::InstanceAlreadyActive),
        );
        assert_eq!(
            conductor.start_instance(&String::from("non-existant-id")),
            Err(HolochainInstanceError::NoSuchInstance),
        );
        assert_eq!(
            conductor.stop_instance(&String::from("test-instance-1")),
            Ok(())
        );
        assert_eq!(
            conductor.stop_instance(&String::from("test-instance-1")),
            Err(HolochainInstanceError::InstanceNotActiveYet),
        );
    }
}
