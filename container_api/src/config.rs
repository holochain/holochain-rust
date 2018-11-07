use holochain_agent::{Agent, Identity};
use holochain_core_types::error::{HcResult, HolochainError};
use holochain_dna::Dna;
use serde::Deserialize;
use std::{fs::File, io::prelude::*};

#[derive(Deserialize, Clone)]
pub struct AgentConfiguration {
    id: String,
    key_file: Option<String>,
}

impl Into<Agent> for AgentConfiguration {
    fn into(self) -> Agent {
        Agent::new(Identity::new(self.id))
    }
}

#[derive(Deserialize)]
pub struct DNAConfiguration {
    id: String,
    file: String,
    hash: String,
}

impl Into<HcResult<Dna>> for DNAConfiguration {
    fn into(self) -> HcResult<Dna> {
        let mut f = File::open(self.file)
            .map_err(|_| HolochainError::IoError(String::from("Could read from file")))?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .map_err(|_| HolochainError::IoError(String::from("Could read from file")))?;
        Dna::from_json_str(&contents)
            .map_err(|_| HolochainError::IoError(String::from("Could not create dna form json")))
    }
}

#[derive(Deserialize)]
pub struct InstanceConfiguration {
    id: String,
    #[serde(rename = "DNA")]
    dna: String,
}

#[derive(Deserialize)]
pub struct Configuration {
    #[serde(rename = "Agents")]
    agents: Option<Vec<AgentConfiguration>>,
    dnas: Option<Vec<DNAConfiguration>>,
}

pub struct LoggerConfiguration {
    logger_type: String,
    file: String,
}

pub struct ContextConfiguration {
    agent: String,
}

#[derive(Deserialize)]
pub struct StorageConfiguration {
    storage_type: String,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    path: Option<String>,
}
#[derive(Deserialize)]
pub struct Bridges {
    id: String,
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
[[Agents]]
id = "bob"
key_file="file/to/serialize"

[[Agents]]
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
