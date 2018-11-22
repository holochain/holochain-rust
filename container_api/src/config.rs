/// Container Configuration
/// This module provides structs that represent the different aspects of how
/// a container can be configured.
/// This mainly means *listing the instances* the container tries to instantiate and run,
/// plus the resources needed by these instances:
/// * agents
/// * DNAs, i.e. the custom app code that makes up the core of a Holochain instance
/// * interfaces, which in this context means ways for user interfaces, either GUIs or local
///   scripts or other local apps, to call DNAs' zome functions and call admin functions of
///   the container
/// * bridges, which are
use boolinator::*;
use holochain_core_types::{
    agent::Agent,
    dna::Dna,
    error::{HcResult, HolochainError},
    json::JsonString,
};
use serde::Deserialize;
use std::{convert::TryFrom, fs::File, io::prelude::*};
use toml;

/// Main container configuration struct
/// This is the root of the configuration tree / aggregates
/// all other configuration aspects.
///
/// References between structs (instance configs pointing to
/// the agent and DNA to be instantiated) are implemented
/// via string IDs.
#[derive(Deserialize, Clone)]
pub struct Configuration {
    /// List of Agents, this mainly means identities and their keys. Required.
    pub agents: Vec<AgentConfiguration>,
    /// List of DNAs, for each a path to the DNA file. Required.
    pub dnas: Vec<DNAConfiguration>,
    /// List of instances, includes references to an agent and a DNA. Required.
    #[serde(default)]
    pub instances: Vec<InstanceConfiguration>,
    /// List of interfaces any UI can use to access zome functions. Optional.
    #[serde(default)]
    pub interfaces: Vec<InterfaceConfiguration>,
    /// List of bridges between instances. Optional.
    #[serde(default)]
    pub bridges: Vec<Bridge>,
}

impl Configuration {
    /// This function basically checks if self is a semantically valid configuration.
    /// This mainly means checking for consistency between config structs that reference others.
    pub fn check_consistency(&self) -> Result<(), String> {
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

        Ok(())
    }

    /// Returns the agent configuration with the given ID if present
    pub fn agent_by_id(&self, id: &String) -> Option<AgentConfiguration> {
        self.agents
            .iter()
            .find(|ac| &ac.id == id)
            .and_then(|agent_config| Some(agent_config.clone()))
    }

    /// Returns the DNA configuration with the given ID if present
    pub fn dna_by_id(&self, id: &String) -> Option<DNAConfiguration> {
        self.dnas
            .iter()
            .find(|dc| &dc.id == id)
            .and_then(|dna_config| Some(dna_config.clone()))
    }

    /// Returns the instance configuration with the given ID if present
    pub fn instance_by_id(&self, id: &String) -> Option<InstanceConfiguration> {
        self.instances
            .iter()
            .find(|ic| &ic.id == id)
            .and_then(|instance_config| Some(instance_config.clone()))
    }

    /// Returns the interface configuration with the given ID if present
    pub fn interface_by_id(&self, id: &String) -> Option<InterfaceConfiguration> {
        self.interfaces
            .iter()
            .find(|ic| &ic.id == id)
            .and_then(|interface_config| Some(interface_config.clone()))
    }

    /// Returns all defined instance IDs
    pub fn instance_ids(&self) -> Vec<String> {
        self.instances
            .iter()
            .map(|instance| instance.id.clone())
            .collect::<Vec<String>>()
    }
}

/// An agent has a name/ID and is defined by a private key that resides in a file
#[derive(Deserialize, Clone)]
pub struct AgentConfiguration {
    pub id: String,
    pub key_file: String,
}

impl From<AgentConfiguration> for Agent {
    fn from(config: AgentConfiguration) -> Self {
        Agent::try_from(JsonString::try_from(config.id).expect("bad agent json"))
            .expect("bad agent json")
    }
}

/// A DNA is represented by a DNA file.
/// A hash has to be provided for sanity check.
#[derive(Deserialize, Clone)]
pub struct DNAConfiguration {
    pub id: String,
    pub file: String,
    pub hash: String,
}

impl TryFrom<DNAConfiguration> for Dna {
    type Error = HolochainError;
    fn try_from(dna_config: DNAConfiguration) -> Result<Self, Self::Error> {
        let mut f = File::open(dna_config.file)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        Dna::try_from(JsonString::from(contents))
    }
}

/// An instance combines a DNA with an agent.
/// Each instance has its own storage and logger configuration.
#[derive(Deserialize, Clone)]
pub struct InstanceConfiguration {
    pub id: String,
    pub dna: String,
    pub agent: String,
    pub logger: LoggerConfiguration,
    pub storage: StorageConfiguration,
}

/// There might be different kinds of loggers in the future.
/// Currently there is no logger at all.
/// TODO: make this an enum when it's actually in use
#[derive(Deserialize, Clone)]
pub struct LoggerConfiguration {
    #[serde(rename = "type")]
    pub logger_type: String,
    pub file: Option<String>,
}

/// This configures the Content Addressable Storage (CAS) that
/// the instance uses to store source chain and DHT shard in.
/// There are two storage implementations in cas_implementations so far:
/// * memory
/// * file
///
/// Projected are various DB adapters.
#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum StorageConfiguration {
    #[serde(rename = "memory")]
    Memory,
    #[serde(rename = "file")]
    File { path: String },
}

/// Here, interfaces are user facing and make available zome functions to
/// GUIs, browser based web UIs, local native UIs, other local applications and scripts.
/// None is implemented yet, but we will have:
/// * websockets
/// * HTTP REST
/// * Unix domain sockets
/// very soon.
///
/// Every interface lists the instances that are made available here.
/// An admin flag will enable container functions for programmatically changing the configuration
/// (i.e. installing apps)
#[derive(Deserialize, Clone)]
pub struct InterfaceConfiguration {
    pub id: String,
    pub driver: InterfaceDriver,
    #[serde(default)]
    pub admin: bool,
    pub instances: Vec<InstanceReferenceConfiguration>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum InterfaceDriver {
    #[serde(rename = "websocket")]
    Websocket { port: u16 },
    #[serde(rename = "http")]
    Http { port: u16 },
    #[serde(rename = "domainsocket")]
    DomainSocket { file: String },
    #[serde(rename = "custom")]
    Custom(toml::value::Value),
}

#[derive(Deserialize, Clone)]
pub struct InstanceReferenceConfiguration {
    pub id: String,
}

/// A bridge enables an instance to call zome functions of another instance.
/// It is basically an internal interface.
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Bridge {
    pub caller_id: String,
    pub callee_id: String,
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

#[cfg(test)]
pub mod tests {

    use config::{load_configuration, Configuration, InterfaceDriver, StorageConfiguration};

    #[test]
    fn test_agent_load() {
        let toml = r#"
    [[agents]]
    id = "bob"
    key_file="file/to/serialize"

    [[agents]]
    id="alex"
    key_file="another/file"

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
                .key_file,
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
    key_file="whatever"

    [[dnas]]
    id = "app spec rust"
    file = "app_spec.hcpkg"
    hash = "Qm328wyq38924y"
    "#;
        let dnas = load_configuration::<Configuration>(toml).unwrap().dnas;
        let dna_config = dnas.get(0).expect("expected at least 1 DNA");
        assert_eq!(dna_config.id, "app spec rust");
        assert_eq!(dna_config.file, "app_spec.hcpkg");
        assert_eq!(dna_config.hash, "Qm328wyq38924y");
    }

    #[test]
    fn test_load_complete_config() {
        let toml = r#"
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
    id = "app spec domainsocket interface"
    [interfaces.driver]
    type = "domainsocket"
    file = "/tmp/holochain.sock"
    [[interfaces.instances]]
    id = "app spec instance"

    "#;
        let config = load_configuration::<Configuration>(toml).unwrap();

        assert_eq!(config.check_consistency(), Ok(()));
        let dnas = config.dnas;
        let dna_config = dnas.get(0).expect("expected at least 1 DNA");
        assert_eq!(dna_config.id, "app spec rust");
        assert_eq!(dna_config.file, "app_spec.hcpkg");
        assert_eq!(dna_config.hash, "Qm328wyq38924y");

        let instances = config.instances;
        let instance_config = instances.get(0).unwrap();
        assert_eq!(instance_config.id, "app spec instance");
        assert_eq!(instance_config.dna, "app spec rust");
        assert_eq!(instance_config.agent, "test agent");
        let logger_config = &instance_config.logger;
        assert_eq!(logger_config.logger_type, "simple");
        assert_eq!(logger_config.file, Some(String::from("app_spec.log")));
        if let StorageConfiguration::File { path } = &instance_config.storage {
            assert_eq!(path, "app_spec_storage");
        } else {
            panic!("Wrong enum type");
        }

        let interfaces = config.interfaces;
        let interface_config_0 = interfaces.get(0).unwrap();
        let interface_config_1 = interfaces.get(1).unwrap();
        if let InterfaceDriver::Websocket { port } = interface_config_0.driver {
            assert_eq!(port, 8888);
        } else {
            panic!("Wrong enum type");
        }
        if let InterfaceDriver::DomainSocket { ref file } = interface_config_1.driver {
            assert_eq!(file, "/tmp/holochain.sock");
        } else {
            panic!("Wrong enum type");
        }
        assert_eq!(interface_config_0.admin, false);
        let instance_ref = interface_config_0.instances.get(0).unwrap();
        assert_eq!(instance_ref.id, "app spec instance");

        assert_eq!(config.bridges, vec![]);
    }

    #[test]
    fn test_inconsistent_config() {
        let toml = r#"
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
    dna = "WRONG DNA ID"
    agent = "test agent"
    [instances.logger]
    type = "simple"
    file = "app_spec.log"
    [instances.storage]
    type = "file"
    path = "app_spec_storage"

    "#;
        let config = load_configuration::<Configuration>(toml).unwrap();

        assert_eq!(config.check_consistency(), Err("DNA configuration \"WRONG DNA ID\" not found, mentioned in instance \"app spec instance\"".to_string()));
    }

    #[test]
    fn test_inconsistent_config_interface_1() {
        let toml = r#"
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
        let toml = r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester"
    key_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app-spec-rust.hcpkg"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app spec instance"
    dna = "app spec rust"
    agent = "test agent"
    [instances.logger]
    type = "simple"
    file = "app_spec.log"
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

    "#;
        if let Err(e) = load_configuration::<Configuration>(toml) {
            assert!(
                true,
                e.to_string().contains("unknown variant `invalid type`")
            )
        } else {
            panic!("Should have failed!")
        }
    }
}
