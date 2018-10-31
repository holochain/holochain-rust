extern crate serde_derive;
extern crate toml;
use holochain_agent::Agent;
use holochain_core_types::error::{HcResult, HolochainError};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename = "Agents")]
struct AgentConfiguration {
    id: String,
    key_file: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename = "DNAs")]
struct DNAConfiguration {
    id: String,
    file: String,
    hash: String,
}

#[derive(Deserialize)]
#[serde(rename = "Instances")]
struct InstanceConfiguration {
    id: String,
    DNA: String,
}

#[derive(Deserialize)]
struct Bridges {
    id: String,
}

fn load_configuration<'a, T>(toml: &'a str) -> HcResult<T>
where
    T: Deserialize<'a>,
{
    toml::from_str(toml)
        .map_err(|_| HolochainError::IoError(String::from("Could not serialize toml")))
}
