extern crate serde_derive;
extern crate toml;
use holochain_core_types::error::{HcResult, HolochainError};
use serde::Deserialize;


pub struct StorageConfiguration {}

#[derive(Deserialize, Clone)]
pub struct AgentConfiguration {
    id: String,
    key_file: Option<String>,
}

#[derive(Deserialize)]
pub struct DNAConfiguration {
    id: String,
    file: String,
    hash: String,
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
