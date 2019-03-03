use crate::{
    config::{
        serialize_configuration, Configuration, InterfaceConfiguration, InterfaceDriver,
        StorageConfiguration,
    },
    context_builder::ContextBuilder,
    error::HolochainInstanceError,
    logger::DebugLogger,
    Holochain,
};
use holochain_common::paths::DNA_EXTENSION;
use holochain_core::{
    logger::{ChannelLogger, Logger},
    signal::Signal,
};
use holochain_core_types::{
    agent::AgentId, cas::content::AddressableContent, dna::Dna, error::HolochainError,
    json::JsonString,
};
use holochain_dpki::{
    key_blob::{Blobbable, KeyBlob},
    key_bundle::KeyBundle,
};
use holochain_sodium::secbuf::SecBuf;
use jsonrpc_ws_server::jsonrpc_core::IoHandler;
use rpassword;
use std::{
    clone::Clone,
    collections::HashMap,
    convert::TryFrom,
    fs::{self, File},
    io::prelude::*,
    path::PathBuf,
    sync::{
        mpsc::{channel, Sender, SyncSender},
        Arc, Mutex, RwLock,
    },
    thread,
};

use holochain_net::{
    ipc::spawn::{ipc_spawn, SpawnResult},
    p2p_config::P2pConfig,
};
use interface::{ConductorApiBuilder, InstanceMap, Interface};
use static_file_server::StaticServer;

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
    agent_keys: HashMap<String, Arc<Mutex<KeyBundle>>>,
    pub(in crate::conductor) config: Configuration,
    pub(in crate::conductor) static_servers: HashMap<String, StaticServer>,
    pub(in crate::conductor) interface_threads: HashMap<String, Sender<()>>,
    pub key_loader: KeyLoader,
    pub(in crate::conductor) dna_loader: DnaLoader,
    pub(in crate::conductor) ui_dir_copier: UiDirCopier,
    signal_tx: Option<SignalSender>,
    logger: DebugLogger,
    p2p_config: Option<P2pConfig>,
    network_spawn: Option<SpawnResult>,
}

impl Drop for Conductor {
    fn drop(&mut self) {
        if let Some(ref mut network_spawn) = self.network_spawn {
            if let Some(kill) = network_spawn.kill.take() {
                kill();
            }
        }
    }
}

type SignalSender = SyncSender<Signal>;
pub type KeyLoader = Arc<Box<FnMut(&PathBuf) -> Result<KeyBundle, HolochainError> + Send + Sync>>;
pub type DnaLoader = Arc<Box<FnMut(&PathBuf) -> Result<Dna, HolochainError> + Send + Sync>>;
pub type UiDirCopier =
    Arc<Box<FnMut(&PathBuf, &PathBuf) -> Result<(), HolochainError> + Send + Sync>>;

// preparing for having conductor notifiers go to one of the log streams
pub fn notify(msg: String) {
    println!("{}", msg);
}

impl Conductor {
    pub fn from_config(config: Configuration) -> Self {
        let rules = config.logger.rules.clone();
        holochain_sodium::check_init();
        Conductor {
            instances: HashMap::new(),
            agent_keys: HashMap::new(),
            interface_threads: HashMap::new(),
            static_servers: HashMap::new(),
            config,
            key_loader: Arc::new(Box::new(Self::load_key)),
            dna_loader: Arc::new(Box::new(Self::load_dna)),
            ui_dir_copier: Arc::new(Box::new(Self::copy_ui_dir)),
            signal_tx: None,
            logger: DebugLogger::new(rules),
            p2p_config: None,
            network_spawn: None,
        }
    }

    pub fn with_signal_channel(mut self, signal_tx: SyncSender<Signal>) -> Self {
        if !self.instances.is_empty() {
            panic!("Cannot set a signal channel after having run load_config()");
        }
        self.signal_tx = Some(signal_tx);
        self
    }

    pub fn p2p_bindings(&self) -> Option<Vec<String>> {
        match self.network_spawn {
            None => None,
            Some(ref spawn) => Some(spawn.p2p_bindings.clone()),
        }
    }

    pub fn config(&self) -> Configuration {
        self.config.clone()
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
            let _ = kill_switch.send(()).map_err(|err| {
                let message = format!("Error stopping interface: {}", err);
                notify(message.clone());
                err
            });
        }
    }

    pub fn stop_interface_by_id(&mut self, id: &String) -> Result<(), HolochainError> {
        {
            let kill_switch =
                self.interface_threads
                    .get(id)
                    .ok_or(HolochainError::ErrorGeneric(format!(
                        "Interface {} not found.",
                        id
                    )))?;
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
        self.config
            .interface_by_id(id)
            .ok_or(format!("Interface does not exist: {}", id))
            .and_then(|config| self.start_interface(&config))
    }

    pub fn start_all_static_servers(&mut self) -> Result<(), String> {
        notify("Starting all servers".into());
        self.static_servers.iter_mut().for_each(|(id, server)| {
            server
                .start()
                .expect(&format!("Couldnt start server {}", id));
            notify(format!("Server started for \"{}\"", id))
        });
        Ok(())
    }

    /// Starts all instances
    pub fn start_all_instances(&mut self) -> Result<(), HolochainInstanceError> {
        self.instances
            .iter_mut()
            .map(|(id, hc)| {
                notify(format!("Starting instance \"{}\"...", id));
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
                notify(format!("Stopping instance \"{}\"...", id));
                hc.write().unwrap().stop()
            })
            .collect::<Result<Vec<()>, _>>()
            .map(|_| ())
    }

    pub fn instances(&self) -> &InstanceMap {
        &self.instances
    }

    /// Stop and clear all instances
    pub fn shutdown(&mut self) {
        let _ = self
            .stop_all_instances()
            .map_err(|error| notify(format!("Error during shutdown: {}", error)));
        self.stop_all_interfaces();
        self.instances = HashMap::new();
    }

    pub fn spawn_network(&mut self) -> Result<SpawnResult, HolochainError> {
        let network_config = self
            .config
            .clone()
            .network
            .ok_or(HolochainError::ErrorGeneric(
                "attempt to spawn network when not configured".to_string(),
            ))?;

        println!(
            "Spawning network with working directory: {}",
            network_config.n3h_persistence_path
        );
        let spawn_result = ipc_spawn(
            "node".to_string(),
            vec![format!(
                "{}/packages/n3h/bin/n3h",
                network_config.n3h_path.clone()
            )],
            network_config.n3h_persistence_path.clone(),
            P2pConfig::load_end_user_config(network_config.networking_config_file).to_string(),
            hashmap! {
                String::from("N3H_MODE") => network_config.n3h_mode.clone(),
                String::from("N3H_WORK_DIR") => network_config.n3h_persistence_path.clone(),
                String::from("N3H_IPC_SOCKET") => String::from("tcp://127.0.0.1:*"),
            },
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

    fn get_p2p_config(&self) -> P2pConfig {
        self.p2p_config.clone().unwrap_or_else(|| {
            // This should never happen, but we'll throw out an in-memory server config rather than crashing,
            // just to be nice (TODO make proper logging statement)
            println!("warn: instance_network_config called before p2p_config initialized! Using default in-memory network name.");
            P2pConfig::new_with_memory_backend("conductor-default-mock")
        })
    }

    fn initialize_p2p_config(&mut self) -> P2pConfig {
        // if there's no NetworkConfig we won't spawn a network process
        // and instead configure instances to use a unique in-memory network
        if let None = self.config.network {
            return P2pConfig::new_with_unique_memory_backend();
        }
        // if there is a config then either we need to spawn a process and get the
        // ipc_uri for it and save it for future calls to `load_config`
        // or we use that uri value that was created from previous calls!
        let net_config = self.config.network.clone().unwrap();
        let uri = match net_config.n3h_ipc_uri.clone() {
            Some(uri) => Some(uri),
            None => {
                self.network_spawn = self.spawn_network().ok();
                match self.network_spawn {
                    None => None,
                    Some(ref spawn) => Some(spawn.ipc_binding.clone()),
                }
            }
        };
        P2pConfig::new_ipc_uri(
            uri,
            &net_config.bootstrap_nodes,
            net_config.networking_config_file,
        )
    }

    /// Tries to create all instances configured in the given Configuration object.
    /// Calls `Configuration::check_consistency()` first and clears `self.instances`.
    /// The first time we call this, we also initialize the conductor-wide config
    /// for use with all instances
    ///
    /// @TODO: clean up the conductor creation process to prevent loading config before proper setup,
    ///        especially regarding the signal handler.
    ///        (see https://github.com/holochain/holochain-rust/issues/739)
    pub fn load_config_with_signal(
        &mut self,
        signal_tx: Option<SignalSender>,
    ) -> Result<(), String> {
        let _ = self.config.check_consistency()?;

        if self.p2p_config.is_none() {
            self.p2p_config = Some(self.initialize_p2p_config());
        }

        let config = self.config.clone();
        self.shutdown();

        for id in config.instance_ids_sorted_by_bridge_dependencies()? {
            let instance = self
                .instantiate_from_config(&id, &config, signal_tx.clone())
                .map_err(|error| {
                    format!(
                        "Error while trying to create instance \"{}\": {}",
                        id, error
                    )
                })?;

            self.instances
                .insert(id.clone(), Arc::new(RwLock::new(instance)));
        }

        for ui_interface_config in config.ui_interfaces.clone() {
            notify(format!("adding ui interface {}", &ui_interface_config.id));
            let bundle_config =
                config
                    .ui_bundle_by_id(&ui_interface_config.bundle)
                    .ok_or(format!(
                        "UI interface {} references bundle with id {} but no such bundle found",
                        &ui_interface_config.id, &ui_interface_config.bundle
                    ))?;
            let connected_dna_interface = ui_interface_config
                .clone()
                .dna_interface
                .map(|interface_id| config.interface_by_id(&interface_id).unwrap());

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

    pub fn load_config(&mut self) -> Result<(), String> {
        self.load_config_with_signal(None)
    }

    /// Creates one specific Holochain instance from a given Configuration,
    /// id string and DnaLoader.
    pub fn instantiate_from_config(
        &mut self,
        id: &String,
        config: &Configuration,
        signal_tx: Option<SignalSender>,
    ) -> Result<Holochain, String> {
        let _ = config.check_consistency()?;

        config
            .instance_by_id(&id)
            .ok_or(String::from("Instance not found in config"))
            .and_then(|instance_config| {
                // Build context:
                let mut context_builder = ContextBuilder::new();

                // Agent:
                let agent_config = config.agent_by_id(&instance_config.agent).unwrap();
                let agent_id = if Some(true) == agent_config.holo_remote_key {
                    // !!!!!!!!!!!!!!!!!!!!!!!
                    // Holo closed-alpha hack:
                    // !!!!!!!!!!!!!!!!!!!!!!!
                    AgentId::new(&agent_config.name, agent_config.public_address)
                } else {
                    let keybundle_arc = self.get_keybundle_for_agent(&instance_config.agent)?;
                    let keybundle = keybundle_arc.lock().unwrap();
                    AgentId::new(&agent_config.name, keybundle.get_id())
                };

                context_builder = context_builder.with_agent(agent_id.clone());

                context_builder = context_builder.with_p2p_config(self.get_p2p_config());

                // Signal config:
                if let Some(tx) = signal_tx {
                    context_builder = context_builder.with_signals(tx)
                };

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
                }

                if config.logger.logger_type == "debug" {
                    context_builder = context_builder.with_logger(Arc::new(Mutex::new(
                        ChannelLogger::new(instance_config.id.clone(), self.logger.get_sender()),
                    )));
                }

                // Conductor API
                let mut api_builder = ConductorApiBuilder::new();
                // Signing callback:
                if Some(true) == agent_config.holo_remote_key {
                    // !!!!!!!!!!!!!!!!!!!!!!!
                    // Holo closed-alpha hack:
                    // !!!!!!!!!!!!!!!!!!!!!!!
                    api_builder = api_builder.with_outsource_signing_callback(
                        agent_id.clone(),
                        self.config
                            .signing_service_uri
                            .clone()
                            .expect("holo_remote_key needs signing_service_uri set"),
                    );
                } else {
                    api_builder = api_builder.with_agent_signature_callback(
                        self.get_keybundle_for_agent(&instance_config.agent)?,
                    );
                }

                // Bridges:
                let id = instance_config.id.clone();
                for bridge in config.bridge_dependencies(id.clone()) {
                    assert_eq!(bridge.caller_id, id.clone());
                    let callee_config = config
                        .instance_by_id(&bridge.callee_id)
                        .expect("config.check_consistency()? jumps out if config is broken");
                    let callee_instance = self.instances.get(&bridge.callee_id).expect(
                        r#"
                    We have to create instances ordered by bridge dependencies such that we
                    can expect the callee to be present here because we need it to create
                    the bridge API"#,
                    );

                    api_builder = api_builder
                        .with_named_instance(bridge.handle.clone(), callee_instance.clone());
                    api_builder = api_builder
                        .with_named_instance_config(bridge.handle.clone(), callee_config);
                }
                context_builder = context_builder.with_conductor_api(api_builder.spawn());
                if let Some(signal_tx) = self.signal_tx.clone() {
                    context_builder = context_builder.with_signals(signal_tx);
                }

                // Spawn context
                let context = context_builder.spawn();

                // Get DNA
                let dna_config = config.dna_by_id(&instance_config.dna).unwrap();
                let dna_file = PathBuf::from(&dna_config.file);
                let dna = Arc::get_mut(&mut self.dna_loader).unwrap()(&dna_file).map_err(|_| {
                    HolochainError::ConfigError(format!(
                        "Could not load DNA file \"{}\"",
                        dna_config.file
                    ))
                })?;

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
                        notify(format!(
                            "Failed to load instance {} from storage: {:?}",
                            id.clone(),
                            loading_error
                        ));
                        notify("Initializing new chain...".to_string());
                        Holochain::new(dna, context).map_err(|hc_err| hc_err.to_string())
                    })
            })
    }

    /// Checks if the key for the given agent can be loaded or was already loaded.
    /// Will trigger loading if key is not loaded yet.
    /// Meant to be used in conductor executable to first try to load all keys (which will trigger
    /// passphrase prompts) before bootstrapping the whole config and have prompts appear
    /// in between other initialization output.
    pub fn check_load_key_for_agent(&mut self, agent_id: &String) -> Result<(), String> {
        if Some(true)
            == self
                .config
                .agent_by_id(agent_id)
                .and_then(|a| a.holo_remote_key)
        {
            // !!!!!!!!!!!!!!!!!!!!!!!
            // Holo closed-alpha hack:
            // !!!!!!!!!!!!!!!!!!!!!!!
            return Ok(());
        }
        self.get_keybundle_for_agent(agent_id)?;
        Ok(())
    }

    /// Get reference to key for given agent ID.
    /// If the key was not loaded (into secure memory) yet, this will use the KeyLoader
    /// to do so.
    fn get_keybundle_for_agent(
        &mut self,
        agent_id: &String,
    ) -> Result<Arc<Mutex<KeyBundle>>, String> {
        if !self.agent_keys.contains_key(agent_id) {
            let agent_config = self
                .config
                .agent_by_id(agent_id)
                .ok_or(format!("Agent '{}' not found", agent_id))?;
            let key_file_path = PathBuf::from(agent_config.key_file.clone());
            let keybundle =
                Arc::get_mut(&mut self.key_loader).unwrap()(&key_file_path).map_err(|_| {
                    HolochainError::ConfigError(format!(
                        "Could not load key file \"{}\"",
                        agent_config.key_file,
                    ))
                })?;
            if agent_config.public_address != keybundle.get_id() {
                return Err(format!(
                    "Key from file '{}' ('{}') does not match public address {} mentioned in config!",
                    key_file_path.to_str().unwrap(),
                    keybundle.get_id(),
                    agent_config.public_address,
                ));
            }
            self.agent_keys
                .insert(agent_id.clone(), Arc::new(Mutex::new(keybundle)));
        }
        let keybundle_ref = self.agent_keys.get(agent_id).unwrap();
        Ok(keybundle_ref.clone())
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
    fn load_dna(file: &PathBuf) -> Result<Dna, HolochainError> {
        notify(format!("Reading DNA from {}", file.display()));
        let mut f = File::open(file)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        Dna::try_from(JsonString::from(contents))
    }

    /// Default KeyLoader that actually reads files from the filesystem
    fn load_key(file: &PathBuf) -> Result<KeyBundle, HolochainError> {
        notify(format!("Reading agent key from {}", file.display()));

        // Read key file
        let mut file = File::open(file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let blob: KeyBlob = serde_json::from_str(&contents)?;

        // Prompt for passphrase
        let mut passphrase_string = rpassword::read_password_from_tty(Some("Passphrase: "))?;

        // Move passphrase in secure memory
        let passphrase_bytes = unsafe { passphrase_string.as_mut_vec() };
        let mut passphrase_buf = SecBuf::with_insecure(passphrase_bytes.len());
        passphrase_buf
            .write(0, passphrase_bytes.as_slice())
            .expect("Failed to write passphrase in a SecBuf");

        // Overwrite the unsafe passphrase memory with zeros
        for byte in passphrase_bytes.iter_mut() {
            *byte = 0u8;
        }

        // Unblob into KeyBundle
        KeyBundle::from_blob(&blob, &mut passphrase_buf, None)
    }

    fn copy_ui_dir(source: &PathBuf, dest: &PathBuf) -> Result<(), HolochainError> {
        notify(format!(
            "Copying UI from {} to {}",
            source.display(),
            dest.display()
        ));
        fs::create_dir_all(dest).map_err(|_| {
            HolochainError::ErrorGeneric(format!("Could not directory structure {:?}", dest).into())
        })?;
        fs_extra::dir::copy(&source, &dest, &fs_extra::dir::CopyOptions::new())
            .map_err(|e| HolochainError::ErrorGeneric(e.to_string()))?;
        Ok(())
    }

    fn make_interface_handler(&self, interface_config: &InterfaceConfiguration) -> IoHandler {
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

        let mut conductor_api_builder = ConductorApiBuilder::new()
            .with_instances(instance_subset)
            .with_instance_configs(self.config.instances.clone());

        if interface_config.admin {
            conductor_api_builder = conductor_api_builder.with_admin_dna_functions();
            conductor_api_builder = conductor_api_builder.with_admin_ui_functions();
        }

        conductor_api_builder.spawn()
    }

    fn spawn_interface_thread(&self, interface_config: InterfaceConfiguration) -> Sender<()> {
        let dispatcher = self.make_interface_handler(&interface_config);
        let log_sender = self.logger.get_sender();
        let (tx, rx) = channel();
        thread::Builder::new()
            .name(format!("conductor-interface: {}", interface_config.id))
            .spawn(move || {
                let iface = make_interface(&interface_config);
                iface.run(dispatcher, rx).map_err(|error| {
                    let message = format!(
                        "err/conductor: Error running interface '{}': {}",
                        interface_config.id, error
                    );
                    let _ = log_sender.send((String::from("conductor"), message));
                    error
                })
            })
            .expect("Could not spawn thread for interface");
        tx
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
            HolochainError::ErrorGeneric(
                format!(
                    "Could not directory structure {:?}",
                    self.config.persistence_dir
                )
                .into(),
            )
        })?;
        let mut file = File::create(&self.config_path()).map_err(|_| {
            HolochainError::ErrorGeneric(
                format!("Could not create file at {:?}", self.config_path()).into(),
            )
        })?;

        file.write(serialize_configuration(&self.config)?.as_bytes())
            .map_err(|_| {
                HolochainError::ErrorGeneric(
                    format!("Could not save config to {:?}", self.config_path()).into(),
                )
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
        serde_json::to_writer_pretty(&file, dna.into())?;
        Ok(path)
    }
}

/// This can eventually be dependency injected for third party Interface definitions
fn make_interface(interface_config: &InterfaceConfiguration) -> Box<Interface> {
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
    extern crate tempfile;
    use crate::config::load_configuration;
    use holochain_core::{
        action::Action, nucleus::actions::call_zome_function::make_cap_request_for_call,
        signal::signal_channel,
    };
    use holochain_core_types::{cas::content::Address, dna, json::RawString};
    use holochain_dpki::{key_bundle::KeyBundle, SEED_SIZE};
    use holochain_sodium::secbuf::SecBuf;
    use holochain_wasm_utils::wasm_target_dir;
    use std::{
        fs::{File, OpenOptions},
        io::Write,
    };

    use self::tempfile::tempdir;
    use test_utils::*;

    pub fn test_dna_loader() -> DnaLoader {
        let loader = Box::new(|path: &PathBuf| {
            Ok(match path.to_str().unwrap().as_ref() {
                "bridge/callee.dna" => callee_dna(),
                "bridge/caller.dna" => caller_dna(),
                _ => Dna::try_from(JsonString::from(example_dna_string())).unwrap(),
            })
        })
            as Box<FnMut(&PathBuf) -> Result<Dna, HolochainError> + Send + Sync>;
        Arc::new(loader)
    }

    pub fn test_key_loader() -> KeyLoader {
        let loader = Box::new(|path: &PathBuf| match path.to_str().unwrap().as_ref() {
            "holo_tester1.key" => Ok(test_keybundle(1)),
            "holo_tester2.key" => Ok(test_keybundle(2)),
            "holo_tester3.key" => Ok(test_keybundle(3)),
            unknown => Err(HolochainError::ErrorGeneric(format!(
                "No test key for {}",
                unknown
            ))),
        })
            as Box<FnMut(&PathBuf) -> Result<KeyBundle, HolochainError> + Send + Sync>;
        Arc::new(loader)
    }

    pub fn test_keybundle(index: u8) -> KeyBundle {
        // Create deterministic seed
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        let mock_seed: Vec<u8> = (1..SEED_SIZE).map(|e| e as u8 + index).collect();
        seed.write(0, mock_seed.as_slice())
            .expect("SecBuf must be writeable");

        // Create KeyBundle from seed
        KeyBundle::new_from_seed_buf(&mut seed, holochain_dpki::seed::SeedType::Mock).unwrap()
    }

    pub fn test_toml() -> String {
        format!(
            r#"
    [[agents]]
    id = "test-agent-1"
    name = "Holo Tester 1"
    public_address = "{}"
    key_file = "holo_tester1.key"

    [[agents]]
    id = "test-agent-2"
    name = "Holo Tester 2"
    public_address = "{}"
    key_file = "holo_tester2.key"

    [[agents]]
    id = "test-agent-3"
    name = "Holo Tester 3"
    public_address = "{}"
    key_file = "holo_tester3.key"

    [[dnas]]
    id = "test-dna"
    file = "app_spec.dna.json"
    hash = "Qm328wyq38924y"

    [[dnas]]
    id = "bridge-callee"
    file = "bridge/callee.dna"
    hash = "Qm328wyq38924y"

    [[dnas]]
    id = "bridge-caller"
    file = "bridge/caller.dna"
    hash = "Qm328wyq38924y"

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
    id = "test-interface"
    admin = true
    [interfaces.driver]
    type = "websocket"
    port = 8888
    [[interfaces.instances]]
    id = "test-instance-1"
    [[interfaces.instances]]
    id = "test-instance-2"

    [[interfaces]]
    id = "test-interface"
    [interfaces.driver]
    type = "http"
    port = 4000
    [[interfaces.instances]]
    id = "test-instance-1"
    [[interfaces.instances]]
    id = "test-instance-2"

    [[bridges]]
    caller_id = "test-instance-2"
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
            test_keybundle(1).get_id(),
            test_keybundle(2).get_id(),
            test_keybundle(3).get_id()
        )
    }

    pub fn test_conductor() -> Conductor {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        conductor.load_config().unwrap();
        conductor
    }

    fn test_conductor_with_signals(signal_tx: SignalSender) -> Conductor {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();
        let mut conductor = Conductor::from_config(config.clone()).with_signal_channel(signal_tx);
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        conductor.load_config().unwrap();
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
                        }
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
    fn test_conductor_load_config() {
        let mut conductor = test_conductor();
        assert_eq!(conductor.instances.len(), 3);

        conductor.start_all_instances().unwrap();
        conductor.start_all_interfaces();
        conductor.stop_all_instances().unwrap();
    }

    //#[test]
    // Default config path ~/.holochain/conductor/conductor-config.toml won't work in CI
    fn _test_conductor_save_and_load_config_default_location() {
        let conductor = test_conductor();
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

    fn callee_dna() -> Dna {
        let wat = &callee_wat();
        let mut dna = create_test_dna_with_wat("greeter", Some(wat));
        dna.uuid = String::from("basic_bridge_call");
        dna.zomes.get_mut("greeter").unwrap().add_fn_declaration(
            String::from("hello"),
            vec![],
            vec![dna::fn_declarations::FnParameter {
                name: String::from("greeting"),
                parameter_type: String::from("String"),
            }],
        );
        dna.zomes
            .get_mut("greeter")
            .unwrap()
            .traits
            .get_mut("hc_public")
            .unwrap()
            .functions
            .push("hello".into());
        dna
    }

    fn caller_dna() -> Dna {
        let wasm = create_wasm_from_file(&format!(
            "{}/wasm32-unknown-unknown/release/test_bridge_caller.wasm",
            wasm_target_dir("conductor_api/", "test-bridge-caller/"),
        ));
        let defs = create_test_defs_with_fn_name("call_bridge");
        let mut dna = create_test_dna_with_defs("test_zome", defs, &wasm);
        dna.uuid = String::from("basic_bridge_call");
        dna
    }

    #[test]
    fn basic_bridge_call_roundtrip() {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        conductor.load_config().expect("Test config must be sane");
        conductor
            .start_all_instances()
            .expect("Instances must be spawnable");
        let caller_instance = conductor.instances["bridge-caller"].clone();
        let mut instance = caller_instance.write().unwrap();

        let cap_call = {
            let context = instance.context();
            make_cap_request_for_call(
                context.clone(),
                Address::from(context.clone().agent_id.address()),
                "call_bridge",
                "{}".to_string(),
            )
        };
        let result = instance
            .call("test_zome", cap_call, "call_bridge", "{}")
            .unwrap();

        // "Holo World" comes for the callee_wat above which runs in the callee instance
        assert_eq!(result, JsonString::from(RawString::from("Holo World")));
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
                key_file = "holo_tester1.key"

                [[dnas]]
                id = "test-dna"
                file = "app_spec.dna.json"
                hash = "Qm328wyq38924y"

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
            conductor.load_config(),
            Err("Error while trying to create instance \"test-instance-1\": Key from file \'holo_tester1.key\' (\'HcSCI7T6wQ5t4nffbjtUk98Dy9fa79Ds6Uzg8nZt8Fyko46ikQvNwfoCfnpuy7z\') does not match public address HoloTester1-----------------------------------------------------------------------AAACZp4xHB mentioned in config!"
                .to_string()),
        );
    }
}
