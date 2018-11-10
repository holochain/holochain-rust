use holochain_core_types::{
    entry::{
        agent::{Agent, Identity},
        dna::Dna,
    },
    error::{HcResult, HolochainError},
    json::JsonString,
};
use serde::Deserialize;
use std::{convert::TryFrom, fs::File, io::prelude::*};
use toml;

#[derive(Deserialize)]
pub struct Configuration {
    agents: Option<Vec<AgentConfiguration>>,
    dnas: Option<Vec<DNAConfiguration>>,
    instances: Option<Vec<InstanceConfiguration>>,
    interfaces: Option<Vec<InterfaceConfiguration>>,
    bridges: Option<Vec<Bridge>>,
}

#[derive(Deserialize, Clone)]
pub struct AgentConfiguration {
    id: String,
    key_file: Option<String>,
}

impl From<AgentConfiguration> for Agent {
    fn from(config: AgentConfiguration) -> Self {
        Agent::from(Identity::from(config.id))
    }
}

#[derive(Deserialize)]
pub struct DNAConfiguration {
    id: String,
    file: String,
    hash: String,
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

#[derive(Deserialize)]
pub struct InstanceConfiguration {
    id: String,
    dna: String,
    agent: String,
    logger: LoggerConfiguration,
    storage: StorageConfiguration,
}

#[derive(Deserialize)]
pub struct LoggerConfiguration {
    #[serde(rename = "type")]
    logger_type: String,
    file: Option<String>,
}

#[derive(Deserialize)]
pub struct StorageConfiguration {
    #[serde(rename = "type")]
    storage_type: String,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    path: Option<String>,
}

#[derive(Deserialize)]
pub struct InterfaceConfiguration {
    #[serde(rename = "type")]
    interface_type: String,
    port: Option<u16>,
    file: Option<String>,
    admin: Option<bool>,
    instances: Vec<InstanceReferenceConfiguration>,
}

#[derive(Deserialize)]
pub struct InstanceReferenceConfiguration {
    id: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Bridge {
    caller_id: String,
    callee_id: String,
}

pub fn load_configuration<'a, T>(toml: &'a str) -> HcResult<T>
where
    T: Deserialize<'a>,
{
    toml::from_str::<T>(toml)
        .map_err(|_| HolochainError::IoError(String::from("Could not serialize toml")))
}

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
