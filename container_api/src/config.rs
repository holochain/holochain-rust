use boolinator::*;
use holochain_core_types::{
    entry::agent::{Agent, Identity},
    error::{HcResult, HolochainError},
    json::JsonString,
};
use holochain_dna::Dna;
use serde::Deserialize;
use std::{convert::TryFrom, fs::File, io::prelude::*};

#[derive(Deserialize)]
pub struct Configuration {
    pub agents: Option<Vec<AgentConfiguration>>,
    pub dnas: Option<Vec<DNAConfiguration>>,
    pub instances: Option<Vec<InstanceConfiguration>>,
    pub interfaces: Option<Vec<InterfaceConfiguration>>,
    pub bridges: Option<Vec<Bridge>>,
}

impl Configuration {
    pub fn check_consistency(&self) -> Result<(), String> {
        if self.instances.is_none() {
            return Err("No instance found".to_string());
        }
        for ref instance in self.instances.as_ref().unwrap().iter() {
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
        if self.interfaces.is_some() {
            for ref interface in self.interfaces.as_ref().unwrap().iter() {
                for ref instance in interface.instances.iter() {
                    self.instance_by_id(&instance.id).is_some().ok_or_else(|| {
                        format!(
                            "Instance configuration \"{}\" not found, mentioned in interface",
                            instance.id
                        )
                    })?;
                }
            }
        }

        Ok(())
    }

    pub fn agent_by_id(&self, id: &String) -> Option<AgentConfiguration> {
        self.agents.as_ref().and_then(|agents| {
            agents
                .iter()
                .find(|ac| &ac.id == id)
                .and_then(|agent_config| Some(agent_config.clone()))
        })
    }

    pub fn dna_by_id(&self, id: &String) -> Option<DNAConfiguration> {
        self.dnas
            .as_ref()
            .and_then(|dnas| dnas.iter().find(|dc| &dc.id == id))
            .and_then(|dna_config| Some(dna_config.clone()))
    }

    pub fn instance_by_id(&self, id: &String) -> Option<InstanceConfiguration> {
        self.instances
            .as_ref()
            .and_then(|instances| instances.iter().find(|ic| &ic.id == id))
            .and_then(|instance_config| Some(instance_config.clone()))
    }

    pub fn instance_ids(&self) -> Vec<String> {
        self.instances
            .as_ref()
            .unwrap()
            .iter()
            .map(|instance| instance.id.clone())
            .collect::<Vec<String>>()
    }
}

#[derive(Deserialize, Clone)]
pub struct AgentConfiguration {
    pub id: String,
    pub key_file: Option<String>,
}

impl From<AgentConfiguration> for Agent {
    fn from(config: AgentConfiguration) -> Self {
        Agent::from(Identity::from(config.id))
    }
}

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

#[derive(Deserialize, Clone)]
pub struct InstanceConfiguration {
    pub id: String,
    pub dna: String,
    pub agent: String,
    pub logger: LoggerConfiguration,
    pub storage: StorageConfiguration,
}

#[derive(Deserialize, Clone)]
pub struct LoggerConfiguration {
    #[serde(rename = "type")]
    pub logger_type: String,
    pub file: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct StorageConfiguration {
    #[serde(rename = "type")]
    pub storage_type: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct InterfaceConfiguration {
    #[serde(rename = "type")]
    pub interface_type: String,
    pub port: Option<u16>,
    pub file: Option<String>,
    pub admin: Option<bool>,
    pub instances: Vec<InstanceReferenceConfiguration>,
}

#[derive(Deserialize)]
pub struct InstanceReferenceConfiguration {
    pub id: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Bridge {
    pub caller_id: String,
    pub callee_id: String,
}

pub fn load_configuration<'a, T>(toml: &'a str) -> HcResult<T>
where
    T: Deserialize<'a>,
{
    toml::from_str::<T>(toml)
        .map_err(|_| HolochainError::IoError(String::from("Could not serialize toml")))
}

mod tests {
    use super::{load_configuration, Configuration};

    #[test]
    fn test_agent_load() {
        let toml = r#"
    [[agents]]
    id = "bob"
    key_file="file/to/serialize"

    [[agents]]
    id="alex"
    "#;
        let agents = load_configuration::<Configuration>(toml)
            .unwrap()
            .agents
            .expect("expected agents returned");
        assert_eq!(agents.get(0).expect("expected at least 2 agents").id, "bob");
        assert_eq!(
            agents
                .get(0)
                .expect("expected at least 2 agents")
                .clone()
                .key_file
                .unwrap(),
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
    [[dnas]]
    id = "app spec rust"
    file = "app-spec-rust.hcpkg"
    hash = "Qm328wyq38924y"
    "#;
        let dnas = load_configuration::<Configuration>(toml)
            .unwrap()
            .dnas
            .expect("expected agents returned");
        let dna_config = dnas.get(0).expect("expected at least 1 DNA");
        assert_eq!(dna_config.id, "app spec rust");
        assert_eq!(dna_config.file, "app-spec-rust.hcpkg");
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
    type = "websocket"
    port = 8888
    [[interfaces.instances]]
    id = "app spec instance"

    "#;
        let config = load_configuration::<Configuration>(toml).unwrap();

        assert_eq!(config.check_consistency(), Ok(()));
        let dnas = config.dnas.expect("expected agents returned");
        let dna_config = dnas.get(0).expect("expected at least 1 DNA");
        assert_eq!(dna_config.id, "app spec rust");
        assert_eq!(dna_config.file, "app-spec-rust.hcpkg");
        assert_eq!(dna_config.hash, "Qm328wyq38924y");

        let instances = config.instances.unwrap();
        let instance_config = instances.get(0).unwrap();
        assert_eq!(instance_config.id, "app spec instance");
        assert_eq!(instance_config.dna, "app spec rust");
        assert_eq!(instance_config.agent, "test agent");
        let logger_config = &instance_config.logger;
        assert_eq!(logger_config.logger_type, "simple");
        assert_eq!(logger_config.file, Some(String::from("app_spec.log")));
        let storage_config = &instance_config.storage;
        assert_eq!(storage_config.storage_type, "file");
        assert_eq!(storage_config.path, Some(String::from("app_spec_storage")));
        assert_eq!(storage_config.username, None);
        assert_eq!(storage_config.password, None);
        assert_eq!(storage_config.url, None);

        let interfaces = config.interfaces.unwrap();
        let interface_config = interfaces.get(0).unwrap();
        assert_eq!(interface_config.interface_type, "websocket");
        assert_eq!(interface_config.port, Some(8888));
        assert_eq!(interface_config.file, None);
        assert_eq!(interface_config.admin, None);
        let instance_ref = interface_config.instances.get(0).unwrap();
        assert_eq!(instance_ref.id, "app spec instance");

        assert_eq!(config.bridges, None);
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
    file = "app-spec-rust.hcpkg"
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
    fn test_inconsistent_config_interface() {
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
}
