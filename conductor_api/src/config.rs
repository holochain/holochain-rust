use crate::logger::LogRules;
/// Conductor Configuration
/// This module provides structs that represent the different aspects of how
/// a conductor can be configured.
/// This mainly means *listing the instances* the conductor tries to instantiate and run,
/// plus the resources needed by these instances:
/// * agents
/// * DNAs, i.e. the custom app code that makes up the core of a Holochain instance
/// * interfaces, which in this context means ways for user interfaces, either GUIs or local
///   scripts or other local apps, to call DNAs' zome functions and call admin functions of
///   the conductor
/// * bridges, which are
use boolinator::*;
use directories;
use holochain_core_types::{
    agent::{AgentId, Base32},
    dna::Dna,
    error::{HcResult, HolochainError},
    json::JsonString,
};
use petgraph::{algo::toposort, graph::DiGraph, prelude::NodeIndex};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    env,
    fs::File,
    io::prelude::*,
    path::PathBuf,
};
use toml;

/// Main conductor configuration struct
/// This is the root of the configuration tree / aggregates
/// all other configuration aspects.
///
/// References between structs (instance configs pointing to
/// the agent and DNA to be instantiated) are implemented
/// via string IDs.
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct Configuration {
    /// List of Agents, this mainly means identities and their keys. Required.
    pub agents: Vec<AgentConfiguration>,
    /// List of DNAs, for each a path to the DNA file. Optional.
    #[serde(default)]
    pub dnas: Vec<DnaConfiguration>,
    /// List of instances, includes references to an agent and a DNA. Optional.
    #[serde(default)]
    pub instances: Vec<InstanceConfiguration>,
    /// List of interfaces any UI can use to access zome functions. Optional.
    #[serde(default)]
    pub interfaces: Vec<InterfaceConfiguration>,
    /// List of bridges between instances. Optional.
    #[serde(default)]
    pub bridges: Vec<Bridge>,
    /// List of ui bundles (static web dirs) to host on a static interface. Optional.
    #[serde(default)]
    pub ui_bundles: Vec<UiBundleConfiguration>,
    /// List of ui interfaces, includes references to ui bundles and dna interfaces it can call. Optional.
    #[serde(default)]
    pub ui_interfaces: Vec<UiInterfaceConfiguration>,
    /// Configures how logging should behave. Optional.
    #[serde(default)]
    pub logger: LoggerConfiguration,
    /// Configuration options for the network module n3h. Optional.
    #[serde(default)]
    pub network: Option<NetworkConfig>,
    /// where to persist the config file and DNAs. Optional.
    #[serde(default = "default_persistence_dir")]
    pub persistence_dir: PathBuf,

    /// Optional URI for a websocket connection to an outsourced signing service.
    /// Bootstrapping step for Holo closed-alpha.
    /// If set, all agents with holo_remote_key = true will be emulated by asking for signatures
    /// over this websocket.
    pub signing_service_uri: Option<String>,

    /// Optional DPKI configuration if conductor is using a DPKI app to initalize and manage
    /// keys for new instances
    pub dpki: Option<DpkiConfiguration>,
}

pub fn default_persistence_dir() -> PathBuf {
    dirs::home_dir()
        .expect("No persistence_dir given in config file and no HOME dir defined. Don't know where to store config file!")
        .join(".holochain")
        .join("conductor")
}

/// There might be different kinds of loggers in the future.
/// Currently there is a "debug" and "simple" logger.
/// TODO: make this an enum
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LoggerConfiguration {
    #[serde(rename = "type")]
    pub logger_type: String,
    #[serde(default)]
    pub rules: LogRules,
    //    pub file: Option<String>,
}

impl Default for LoggerConfiguration {
    fn default() -> LoggerConfiguration {
        LoggerConfiguration {
            logger_type: "debug".into(),
            rules: Default::default(),
        }
    }
}

/// Check for duplicate items in a list of strings
fn detect_dupes<'a, I: Iterator<Item = &'a String>>(
    name: &'static str,
    items: I,
) -> Result<(), String> {
    let mut set = HashSet::<&str>::new();
    let mut dupes = Vec::<String>::new();
    for item in items {
        if !set.insert(item) {
            dupes.push(item.to_string())
        }
    }
    if !dupes.is_empty() {
        Err(format!(
            "Duplicate {} IDs detected: {}",
            name,
            dupes.join(", ")
        ))
    } else {
        Ok(())
    }
}

impl Configuration {
    /// This function basically checks if self is a semantically valid configuration.
    /// This mainly means checking for consistency between config structs that reference others.
    pub fn check_consistency<'a>(&'a self) -> Result<(), String> {
        detect_dupes("agent", self.agents.iter().map(|c| &c.id))?;
        detect_dupes("dna", self.dnas.iter().map(|c| &c.id))?;
        detect_dupes("instance", self.instances.iter().map(|c| &c.id))?;
        detect_dupes("interface", self.interfaces.iter().map(|c| &c.id))?;

        for ref instance in self.instances.iter() {
            self.agent_by_id(&instance.agent).is_some().ok_or_else(|| {
                format!(
                    "Agent configuration {} not found, mentioned in instance {}",
                    instance.agent, instance.id
                )
            })?;
            self.dna_by_id(&instance.dna).is_some().ok_or_else(|| {
                format!(
                    "DNA configuration \"{}\" not found, mentioned in instance \"{}\"",
                    instance.dna, instance.id
                )
            })?;
        }
        for ref interface in self.interfaces.iter() {
            for ref instance in interface.instances.iter() {
                self.instance_by_id(&instance.id).is_some().ok_or_else(|| {
                    format!(
                        "Instance configuration \"{}\" not found, mentioned in interface",
                        instance.id
                    )
                })?;
            }
        }

        for ref bridge in self.bridges.iter() {
            self.instance_by_id(&bridge.callee_id)
                .is_some()
                .ok_or_else(|| {
                    format!(
                        "Instance configuration \"{}\" not found, mentioned in bridge",
                        bridge.callee_id
                    )
                })?;
            self.instance_by_id(&bridge.caller_id)
                .is_some()
                .ok_or_else(|| {
                    format!(
                        "Instance configuration \"{}\" not found, mentioned in bridge",
                        bridge.caller_id
                    )
                })?;
        }

        for ref ui_interface in self.ui_interfaces.iter() {
            self.ui_bundle_by_id(&ui_interface.bundle)
                .is_some()
                .ok_or_else(|| {
                    format!(
                        "UI bundle configuration {} not found, mentioned in UI interface {}",
                        ui_interface.bundle, ui_interface.id,
                    )
                })?;

            if let Some(ref dna_interface_id) = ui_interface.dna_interface {
                self.interface_by_id(&dna_interface_id)
                    .is_some()
                    .ok_or_else(|| {
                        format!(
                            "DNA Interface configuration \"{}\" not found, mentioned in UI interface \"{}\"",
                            dna_interface_id, ui_interface.id,
                        )
                    })?;
            }
        }

        if let Some(ref dpki_config) = self.dpki {
            self.instance_by_id(&dpki_config.instance_id)
                .is_some()
                .ok_or_else(|| {
                    format!(
                        "Instance configuration \"{}\" not found, mentioned in dpki",
                        dpki_config.instance_id
                    )
                })?;
        }

        let _ = self.instance_ids_sorted_by_bridge_dependencies()?;

        Ok(())
    }

    /// Returns the agent configuration with the given ID if present
    pub fn agent_by_id(&self, id: &str) -> Option<AgentConfiguration> {
        self.agents.iter().find(|ac| &ac.id == id).cloned()
    }

    /// Returns the DNA configuration with the given ID if present
    pub fn dna_by_id(&self, id: &str) -> Option<DnaConfiguration> {
        self.dnas.iter().find(|dc| &dc.id == id).cloned()
    }

    /// Returns the instance configuration with the given ID if present
    pub fn instance_by_id(&self, id: &str) -> Option<InstanceConfiguration> {
        self.instances.iter().find(|ic| &ic.id == id).cloned()
    }

    /// Returns the interface configuration with the given ID if present
    pub fn interface_by_id(&self, id: &str) -> Option<InterfaceConfiguration> {
        self.interfaces.iter().find(|ic| &ic.id == id).cloned()
    }

    pub fn ui_bundle_by_id(&self, id: &str) -> Option<UiBundleConfiguration> {
        self.ui_bundles.iter().find(|ic| &ic.id == id).cloned()
    }

    /// Returns all defined instance IDs
    pub fn instance_ids(&self) -> Vec<String> {
        self.instances
            .iter()
            .map(|instance| instance.id.clone())
            .collect()
    }

    /// This function uses the petgraph crate to model the bridge connections in this config
    /// as a graph and then create a topological sorting of the nodes, which are instances.
    /// The sorting gets reversed to get those instances first that do NOT depend on others
    /// such that this ordering of instances can be used to spawn them and simultaneously create
    /// initialize the bridges and be able to assert that any callee already exists (which makes
    /// this task much easier).
    pub fn instance_ids_sorted_by_bridge_dependencies(
        &self,
    ) -> Result<Vec<String>, HolochainError> {
        let mut graph = DiGraph::<&str, &str>::new();

        // Add instance ids to the graph which returns the indices the graph is using.
        // Storing those in a map from ids to create edges from bridges below.
        let index_map: HashMap<_, _> = self
            .instances
            .iter()
            .map(|instance| (instance.id.clone(), graph.add_node(&instance.id)))
            .collect();

        // Reverse of graph indices to instance ids to create the return vector below.
        let reverse_map: HashMap<_, _> = self
            .instances
            .iter()
            .map(|instance| (index_map.get(&instance.id).unwrap(), instance.id.clone()))
            .collect();

        // Create vector of edges (with node indices) from bridges:
        let edges: Vec<(&NodeIndex<u32>, &NodeIndex<u32>)> = self.bridges
            .iter()
            .map(|bridge| -> Result<(&NodeIndex<u32>, &NodeIndex<u32>), HolochainError> {
                let start = index_map.get(&bridge.caller_id);
                let end = index_map.get(&bridge.callee_id);
                if start.is_none() || end.is_none() {
                    Err(HolochainError::ConfigError(format!(
                        "Instance configuration not found, mentioned in bridge configuration: {} -> {}",
                        bridge.caller_id, bridge.callee_id,
                    )))
                } else {
                    Ok((start.unwrap(), end.unwrap()))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Add edges to graph:
        for &(node_a, node_b) in edges.iter() {
            graph.add_edge(node_a.clone(), node_b.clone(), "");
        }

        // Sort with petgraph::algo::toposort
        let mut sorted_nodes = toposort(&graph, None).map_err(|_cycle_error| {
            HolochainError::ConfigError("Cyclic dependency in bridge configuration".to_string())
        })?;

        // REVERSE order because we want to get the instance with NO dependencies first
        // since that is the instance we should spawn first.
        sorted_nodes.reverse();

        // Map sorted vector of node indices back to instance ids
        Ok(sorted_nodes
            .iter()
            .map(|node_index| reverse_map.get(node_index).unwrap())
            .cloned()
            .collect())
    }

    pub fn bridge_dependencies(&self, caller_instance_id: String) -> Vec<Bridge> {
        self.bridges
            .iter()
            .filter(|bridge| bridge.caller_id == caller_instance_id)
            .cloned()
            .collect()
    }

    /// Removes the instance given by id and all mentions of it in other elements so
    /// that the config is guaranteed to be valid afterwards if it was before.
    pub fn save_remove_instance(mut self, id: &String) -> Self {
        self.instances = self
            .instances
            .into_iter()
            .filter(|instance| instance.id != *id)
            .collect();

        self.interfaces = self
            .interfaces
            .into_iter()
            .map(|mut interface| {
                interface.instances = interface
                    .instances
                    .into_iter()
                    .filter(|instance| instance.id != *id)
                    .collect();
                interface
            })
            .collect();

        self
    }
}

/// An agent has a name/ID and is defined by a private key that resides in a file
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct AgentConfiguration {
    pub id: String,
    pub name: String,
    pub public_address: Base32,
    pub keystore_file: String,
    /// If set to true conductor will ignore keystore_file and instead use the remote signer
    /// accessible through signing_service_uri to request signatures.
    pub holo_remote_key: Option<bool>,
}

impl From<AgentConfiguration> for AgentId {
    fn from(config: AgentConfiguration) -> Self {
        AgentId::try_from(JsonString::from_json(&config.id)).expect("bad agent json")
    }
}

/// A DNA is represented by a DNA file.
/// A hash can optionally be provided, which could be used to validate that the DNA being installed
/// is the DNA that was intended to be installed.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct DnaConfiguration {
    pub id: String,
    pub file: String,
    #[serde(default)]
    pub hash: Option<String>,
}

impl TryFrom<DnaConfiguration> for Dna {
    type Error = HolochainError;
    fn try_from(dna_config: DnaConfiguration) -> Result<Self, Self::Error> {
        let mut f = File::open(dna_config.file)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        Dna::try_from(JsonString::from_json(&contents))
    }
}

/// An instance combines a DNA with an agent.
/// Each instance has its own storage configuration.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct InstanceConfiguration {
    pub id: String,
    pub dna: String,
    pub agent: String,
    pub storage: StorageConfiguration,
}

/// This configures the Content Addressable Storage (CAS) that
/// the instance uses to store source chain and DHT shard in.
/// There are two storage implementations in cas_implementations so far:
/// * memory
/// * file
///
/// Projected are various DB adapters.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StorageConfiguration {
    Memory,
    File { path: String },
    Pickle { path: String },
}

/// Here, interfaces are user facing and make available zome functions to
/// GUIs, browser based web UIs, local native UIs, other local applications and scripts.
/// We currently have:
/// * websockets
/// * HTTP
///
/// We will also soon develop
/// * Unix domain sockets
///
/// The instances (referenced by ID) that are to be made available via that interface should be listed.
/// An admin flag will enable conductor functions for programatically changing the configuration
/// (e.g. installing apps)
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct InterfaceConfiguration {
    pub id: String,
    pub driver: InterfaceDriver,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub instances: Vec<InstanceReferenceConfiguration>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InterfaceDriver {
    Websocket { port: u16 },
    Http { port: u16 },
    DomainSocket { file: String },
    Custom(toml::value::Value),
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct InstanceReferenceConfiguration {
    pub id: String,
}

/// A bridge enables an instance to call zome functions of another instance.
/// It is basically an internal interface.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Bridge {
    /// ID of the instance that calls the other one.
    /// This instance depends on the callee.
    pub caller_id: String,

    /// ID of the instance that exposes traits through this bridge.
    /// This instance is used by the caller.
    pub callee_id: String,

    /// The caller's local handle of this bridge and the callee.
    /// A caller can have many bridges to other DNAs and those DNAs could
    /// by bound dynamically.
    /// Callers reference callees by this arbitrary but unique local name.
    pub handle: String,
}

/// A UI Bundle is a folder containing static assets which can be served as a UI
/// A hash can optionally be provided, which could be used to validate that the UI being installed
/// is the UI bundle that was intended to be installed.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct UiBundleConfiguration {
    pub id: String,
    pub root_dir: String,
    #[serde(default)]
    pub hash: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct UiInterfaceConfiguration {
    pub id: String,

    /// ID of the bundle to serve on this interface
    pub bundle: String,
    pub port: u16,

    /// DNA interface this UI is allowed to make calls to
    /// This is used to set the CORS headers and also to
    /// provide a extra virtual file endpoint at /_dna_config/ that allows hc-web-client
    /// or another solution to redirect holochain calls to the correct ip/port/protocol
    /// (Optional)
    #[serde(default)]
    pub dna_interface: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NetworkConfig {
    /// List of URIs that point to other nodes to bootstrap p2p connections.
    #[serde(default)]
    pub bootstrap_nodes: Vec<String>,
    /// Global logging level output by N3H
    #[serde(default = "default_n3h_log_level")]
    pub n3h_log_level: String,
    /// Absolute path to the local installation/repository of n3h
    #[serde(default)]
    pub n3h_path: String,
    /// networking mode used by n3h
    #[serde(default = "default_n3h_mode")]
    pub n3h_mode: String,
    /// Absolute path to the directory that n3h uses to store persisted data.
    #[serde(default)]
    pub n3h_persistence_path: String,
    /// URI pointing to an n3h process that is already running and not managed by this
    /// conductor.
    /// If this is set the conductor does not spawn n3h itself and ignores the path
    /// configs above. Default is None.
    #[serde(default)]
    pub n3h_ipc_uri: Option<String>,
    /// filepath to the json file holding the network settings for n3h
    #[serde(default)]
    pub networking_config_file: Option<String>,
}

// note that this behaviour is documented within
// holochain_common::env_vars module and should be updated
// if this logic changes
pub fn default_n3h_mode() -> String {
    String::from("HACK")
}

// note that this behaviour is documented within
// holochain_common::env_vars module and should be updated
// if this logic changes
pub fn default_n3h_log_level() -> String {
    String::from("i")
}

// note that this behaviour is documented within
// holochain_common::env_vars module and should be updated
// if this logic changes
pub fn default_n3h_path() -> String {
    if let Some(user_dirs) = directories::UserDirs::new() {
        user_dirs
            .home_dir()
            .join(".hc")
            .join("net")
            .join("n3h")
            .to_string_lossy()
            .to_string()
    } else {
        String::from("n3h")
    }
}

// note that this behaviour is documented within
// holochain_common::env_vars module and should be updated
// if this logic changes
pub fn default_n3h_persistence_path() -> String {
    env::temp_dir().to_string_lossy().to_string()
}

/// Use this function to load a `Configuration` from a string.
pub fn load_configuration<'a, T>(toml: &'a str) -> HcResult<T>
where
    T: Deserialize<'a>,
{
    toml::from_str::<T>(toml).map_err(|e| {
        HolochainError::IoError(format!("Could not serialize toml: {}", e.to_string()))
    })
}

pub fn serialize_configuration(config: &Configuration) -> HcResult<String> {
    // see https://github.com/alexcrichton/toml-rs/issues/142
    let config_toml = toml::Value::try_from(config).map_err(|e| {
        HolochainError::IoError(format!("Could not serialize toml: {}", e.to_string()))
    })?;
    toml::to_string_pretty(&config_toml).map_err(|e| {
        HolochainError::IoError(format!(
            "Could not convert toml to string: {}",
            e.to_string()
        ))
    })
}

/// Configure which app instance id to treat as the DPKI application handler
/// as well as what parameters to pass it on its initialization
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct DpkiConfiguration {
    pub instance_id: String,
    pub init_params: String,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::config::{load_configuration, Configuration, NetworkConfig};
    use holochain_net::p2p_config::P2pConfig;

    pub fn example_serialized_network_config() -> String {
        String::from(JsonString::from(P2pConfig::new_with_unique_memory_backend()))
    }

    #[test]
    fn test_agent_load() {
        let toml = r#"
    [[agents]]
    id = "bob"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "file/to/serialize"

    [[agents]]
    id="alex"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "another/file"

    [[dnas]]
    id="dna"
    file="file.dna.json"
    hash="QmDontCare"
    "#;
        let agents = load_configuration::<Configuration>(toml).unwrap().agents;
        assert_eq!(agents.get(0).expect("expected at least 2 agents").id, "bob");
        assert_eq!(
            agents
                .get(0)
                .expect("expected at least 2 agents")
                .clone()
                .keystore_file,
            "file/to/serialize"
        );
        assert_eq!(
            agents.get(1).expect("expected at least 2 agents").id,
            "alex"
        );
    }

    #[test]
    fn test_dna_load() {
        let toml = r#"
    [[agents]]
    id="agent"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "whatever"

    [[dnas]]
    id = "app spec rust"
    file = "app_spec.dna.json"
    hash = "Qm328wyq38924y"
    "#;
        let dnas = load_configuration::<Configuration>(toml).unwrap().dnas;
        let dna_config = dnas.get(0).expect("expected at least 1 DNA");
        assert_eq!(dna_config.id, "app spec rust");
        assert_eq!(dna_config.file, "app_spec.dna.json");
        assert_eq!(dna_config.hash, Some("Qm328wyq38924y".to_string()));
    }

    #[test]
    fn test_load_complete_config() {
        let toml = r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app_spec.dna.json"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app spec instance"
    dna = "app spec rust"
    agent = "test agent"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    [[interfaces]]
    id = "app spec websocket interface"
    [interfaces.driver]
    type = "websocket"
    port = 8888
    [[interfaces.instances]]
    id = "app spec instance"

    [[interfaces]]
    id = "app spec http interface"
    [interfaces.driver]
    type = "http"
    port = 4000
    [[interfaces.instances]]
    id = "app spec instance"

    [[interfaces]]
    id = "app spec domainsocket interface"
    [interfaces.driver]
    type = "domainsocket"
    file = "/tmp/holochain.sock"
    [[interfaces.instances]]
    id = "app spec instance"

    [network]
    bootstrap_nodes = ["wss://192.168.0.11:64519/?a=hkYW7TrZUS1hy-i374iRu5VbZP1sSw2mLxP4TSe_YI1H2BJM3v_LgAQnpmWA_iR1W5k-8_UoA1BNjzBSUTVNDSIcz9UG0uaM"]
    n3h_path = "/Users/cnorris/.holochain/n3h"
    n3h_persistence_path = "/Users/cnorris/.holochain/n3h_persistence"
    networking_config_file = "/Users/cnorris/.holochain/network_config.json"
    n3h_log_level = "d"
    "#;

        let config = load_configuration::<Configuration>(toml).unwrap();

        assert_eq!(config.check_consistency(), Ok(()));
        let dnas = config.dnas;
        let dna_config = dnas.get(0).expect("expected at least 1 DNA");
        assert_eq!(dna_config.id, "app spec rust");
        assert_eq!(dna_config.file, "app_spec.dna.json");
        assert_eq!(dna_config.hash, Some("Qm328wyq38924y".to_string()));

        let instances = config.instances;
        let instance_config = instances.get(0).unwrap();
        assert_eq!(instance_config.id, "app spec instance");
        assert_eq!(instance_config.dna, "app spec rust");
        assert_eq!(instance_config.agent, "test agent");
        assert_eq!(config.logger.logger_type, "debug");
        assert_eq!(
            config.network.unwrap(),
            NetworkConfig {
                bootstrap_nodes: vec![String::from(
                    "wss://192.168.0.11:64519/?a=hkYW7TrZUS1hy-i374iRu5VbZP1sSw2mLxP4TSe_YI1H2BJM3v_LgAQnpmWA_iR1W5k-8_UoA1BNjzBSUTVNDSIcz9UG0uaM"
                )],
                n3h_log_level: String::from("d"),
                n3h_path: String::from("/Users/cnorris/.holochain/n3h"),
                n3h_mode: String::from("HACK"),
                n3h_persistence_path: String::from("/Users/cnorris/.holochain/n3h_persistence"),
                n3h_ipc_uri: None,
                networking_config_file: Some(String::from(
                    "/Users/cnorris/.holochain/network_config.json"
                )),
            }
        );
    }

    #[test]
    fn test_load_complete_config_default_network() {
        let toml = r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app_spec.dna.json"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app spec instance"
    dna = "app spec rust"
    agent = "test agent"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    [[interfaces]]
    id = "app spec websocket interface"
    [interfaces.driver]
    type = "websocket"
    port = 8888
    [[interfaces.instances]]
    id = "app spec instance"

    [[interfaces]]
    id = "app spec http interface"
    [interfaces.driver]
    type = "http"
    port = 4000
    [[interfaces.instances]]
    id = "app spec instance"

    [[interfaces]]
    id = "app spec domainsocket interface"
    [interfaces.driver]
    type = "domainsocket"
    file = "/tmp/holochain.sock"
    [[interfaces.instances]]
    id = "app spec instance"

    [logger]
    type = "debug"
    [[logger.rules.rules]]
    pattern = ".*"
    color = "red"

    [[ui_bundles]]
    id = "bundle1"
    root_dir = "" # serves the current directory
    hash = "Qm000"

    [[ui_interfaces]]
    id = "ui-interface-1"
    bundle = "bundle1"
    port = 3000
    dna_interface = "app spec domainsocket interface"

    "#;

        let config = load_configuration::<Configuration>(toml).unwrap();

        assert_eq!(config.check_consistency(), Ok(()));
        let dnas = config.dnas;
        let dna_config = dnas.get(0).expect("expected at least 1 DNA");
        assert_eq!(dna_config.id, "app spec rust");
        assert_eq!(dna_config.file, "app_spec.dna.json");
        assert_eq!(dna_config.hash, Some("Qm328wyq38924y".to_string()));

        let instances = config.instances;
        let instance_config = instances.get(0).unwrap();
        assert_eq!(instance_config.id, "app spec instance");
        assert_eq!(instance_config.dna, "app spec rust");
        assert_eq!(instance_config.agent, "test agent");
        assert_eq!(config.logger.logger_type, "debug");
        assert_eq!(config.logger.rules.rules.len(), 1);

        assert_eq!(config.network, None);
    }

    #[test]
    fn test_inconsistent_config() {
        let toml = r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app_spec.dna.json"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app spec instance"
    dna = "WRONG DNA ID"
    agent = "test agent"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    "#;

        let config: Configuration =
            load_configuration(toml).expect("Failed to load config from toml string");

        assert_eq!(config.check_consistency(), Err("DNA configuration \"WRONG DNA ID\" not found, mentioned in instance \"app spec instance\"".to_string()));
    }

    #[test]
    fn test_inconsistent_config_interface_1() {
        let toml = r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app_spec.dna.json"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app spec instance"
    dna = "app spec rust"
    agent = "test agent"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    [[interfaces]]
    id = "app spec interface"
    [interfaces.driver]
    type = "websocket"
    port = 8888
    [[interfaces.instances]]
    id = "WRONG INSTANCE ID"
    "#;

        let config = load_configuration::<Configuration>(toml).unwrap();

        assert_eq!(
            config.check_consistency(),
            Err(
                "Instance configuration \"WRONG INSTANCE ID\" not found, mentioned in interface"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_invalid_toml_1() {
        let toml = &format!(
            r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app-spec-rust.dna.json"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app spec instance"
    dna = "app spec rust"
    agent = "test agent"
    network = "{}"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    [[interfaces]]
    id = "app spec interface"
    [interfaces.driver]
    type = "invalid type"
    port = 8888
    [[interfaces.instances]]
    id = "app spec instance"
    "#,
            example_serialized_network_config()
        );
        if let Err(e) = load_configuration::<Configuration>(toml) {
            assert!(
                true,
                e.to_string().contains("unknown variant `invalid type`")
            )
        } else {
            panic!("Should have failed!")
        }
    }

    fn bridges_config(bridges: &str) -> String {
        format!(
            r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app-spec-rust.dna.json"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app1"
    dna = "app spec rust"
    agent = "test agent"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    [[instances]]
    id = "app2"
    dna = "app spec rust"
    agent = "test agent"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    [[instances]]
    id = "app3"
    dna = "app spec rust"
    agent = "test agent"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    {}
    "#, bridges)
    }

    #[test]
    fn test_bridge_config() {
        let toml = bridges_config(
            r#"
    [[bridges]]
    caller_id = "app1"
    callee_id = "app2"
    handle = "happ-store"

    [[bridges]]
    caller_id = "app2"
    callee_id = "app3"
    handle = "DPKI"
    "#,
        );
        let config = load_configuration::<Configuration>(&toml)
            .expect("Config should be syntactically correct");
        assert_eq!(config.check_consistency(), Ok(()));

        // "->": calls
        // app1 -> app2 -> app3
        // app3 has no dependency so it can be instantiated first.
        // app2 depends on (calls) only app3, so app2 is next.
        // app1 should be last.
        assert_eq!(
            config.instance_ids_sorted_by_bridge_dependencies(),
            Ok(vec![
                String::from("app3"),
                String::from("app2"),
                String::from("app1")
            ])
        );
    }

    #[test]
    fn test_bridge_cycle() {
        let toml = bridges_config(
            r#"
    [[bridges]]
    caller_id = "app1"
    callee_id = "app2"
    handle = "happ-store"

    [[bridges]]
    caller_id = "app2"
    callee_id = "app3"
    handle = "DPKI"

    [[bridges]]
    caller_id = "app3"
    callee_id = "app1"
    handle = "something"
    "#,
        );
        let config = load_configuration::<Configuration>(&toml)
            .expect("Config should be syntactically correct");
        assert_eq!(
            config.check_consistency(),
            Err("Cyclic dependency in bridge configuration".to_string())
        );
    }

    #[test]
    fn test_bridge_non_existent() {
        let toml = bridges_config(
            r#"
    [[bridges]]
    caller_id = "app1"
    callee_id = "app2"
    handle = "happ-store"

    [[bridges]]
    caller_id = "app2"
    callee_id = "app3"
    handle = "DPKI"

    [[bridges]]
    caller_id = "app9000"
    callee_id = "app1"
    handle = "something"
    "#,
        );
        let config = load_configuration::<Configuration>(&toml)
            .expect("Config should be syntactically correct");
        assert_eq!(
            config.check_consistency(),
            Err("Instance configuration \"app9000\" not found, mentioned in bridge".to_string())
        );
    }

    #[test]
    fn test_bridge_dependencies() {
        let toml = bridges_config(
            r#"
    [[bridges]]
    caller_id = "app1"
    callee_id = "app2"
    handle = "happ-store"

    [[bridges]]
    caller_id = "app1"
    callee_id = "app3"
    handle = "happ-store"

    [[bridges]]
    caller_id = "app2"
    callee_id = "app1"
    handle = "happ-store"
    "#,
        );
        let config = load_configuration::<Configuration>(&toml)
            .expect("Config should be syntactically correct");
        let bridged_ids: Vec<_> = config
            .bridge_dependencies(String::from("app1"))
            .iter()
            .map(|bridge| bridge.callee_id.clone())
            .collect();
        assert_eq!(
            bridged_ids,
            vec![String::from("app2"), String::from("app3"),]
        );
    }

    #[test]
    fn test_n3h_defaults() {
        assert_eq!(default_n3h_mode(), String::from("HACK"));

        #[cfg(not(windows))]
        assert!(default_n3h_path().contains("/.hc/net/n3h"));

        // the path can be lots of things in different environments (travis CI etc)
        // so we are just testing that it isn't null
        #[cfg(not(windows))]
        assert!(default_n3h_persistence_path() != String::from(""));
    }

    #[test]
    fn test_inconsistent_ui_interface() {
        let toml = r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app_spec.dna.json"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app spec instance"
    dna = "app spec rust"
    agent = "test agent"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    [[interfaces]]
    id = "app spec websocket interface"
    [interfaces.driver]
    type = "websocket"
    port = 8888
    [[interfaces.instances]]
    id = "app spec instance"

    [[interfaces]]
    id = "app spec http interface"
    [interfaces.driver]
    type = "http"
    port = 4000
    [[interfaces.instances]]
    id = "app spec instance"

    [[interfaces]]
    id = "app spec domainsocket interface"
    [interfaces.driver]
    type = "domainsocket"
    file = "/tmp/holochain.sock"
    [[interfaces.instances]]
    id = "app spec instance"

    [logger]
    type = "debug"
    [[logger.rules.rules]]
    pattern = ".*"
    color = "red"

    [[ui_bundles]]
    id = "bundle1"
    root_dir = "" # serves the current directory
    hash = "Qm000"

    [[ui_interfaces]]
    id = "ui-interface-1"
    bundle = "bundle1"
    port = 3000
    dna_interface = "<not existant>"

    "#;
        let config = load_configuration::<Configuration>(&toml)
            .expect("Config should be syntactically correct");
        assert_eq!(
            config.check_consistency(),
            Err("DNA Interface configuration \"<not existant>\" not found, mentioned in UI interface \"ui-interface-1\"".to_string())
        );
    }

    #[test]
    fn test_inconsistent_dpki() {
        let toml = r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester 1"
    public_address = "HoloTester1-------------------------------------------------------------------------AHi1"
    keystore_file = "holo_tester.key"

    [[dnas]]
    id = "deepkey"
    file = "deepkey.dna.json"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "deepkey"
    dna = "deepkey"
    agent = "test agent"
    [instances.storage]
    type = "file"
    path = "deepkey_storage"

    [dpki]
    instance_id = "bogus instance"
    init_params = "{}"

    "#;
        let config = load_configuration::<Configuration>(&toml)
            .expect("Config should be syntactically correct");
        assert_eq!(
            config.check_consistency(),
            Err(
                "Instance configuration \"bogus instance\" not found, mentioned in dpki"
                    .to_string()
            )
        );
    }
}
