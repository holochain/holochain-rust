use holochain_cas_implementations::{
    cas::{file::FilesystemStorage, memory::MemoryStorage},
    eav::memory::EavMemoryStorage,
};
use holochain_container_api::{
    config::{AgentConfiguration, Configuration, DNAConfiguration, InstanceConfiguration, StorageConfiguration},
    Holochain,
};
use holochain_core::{
    context::{mock_network_config, Context as HolochainContext},
    logger::Logger,
    persister::SimplePersister,
};
use holochain_core_types::{agent::AgentId, dna::Dna, json::JsonString};
use neon::{context::Context, prelude::*};
use std::{
    convert::TryFrom,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};
use tempfile::tempdir;

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

pub struct App {
    instance: Holochain,
}

pub struct HcTest {}

#[derive(Serialize, Deserialize)]
pub struct AgentData {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct DnaData {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct InstanceData {
    pub agent: AgentData,
    pub dna: DnaData,
}

pub struct ScenarioConfig(pub Vec<InstanceData>);

pub fn make_config(instance_data: Vec<InstanceData>) -> Result<Configuration, String> {
    let agent_configs = HashMap::new();
    let dna_configs = HashMap::new();
    let instance_configs = Vec::new();
    for instance in instance_data {
        let agent_name = instance.agent.name;
        let dna_path = instance.dna.path;
        let agent = agent_configs
            .entry(agent_name)
            .insert_or(AgentConfiguration {
                id: agent_name,
                key_file: "DONTCARE".into(),
            });
        let dna = dna_configs
            .entry(dna_path)
            .insert_or_with(|| make_dna_config(dna_path)?);
        
        let logger = {
            logger_type: "DONTCARE".into(),
            file: None,
        };
        let instance = InstanceConfiguration {
            id: "TODO",
            dna: dna.id,
            agent: agent.id,
            logger,
            storage: StorageConfiguration::Memory,
            network: None,
        };
        instance_configs.push(instance);
    }

    const config = Configuration {
        agents: agent_configs.values(),
        dnas: dna_configs.values(),
        instances: instance_configs,
        interfaces: Vec::new(),
        bridges: Vec::new(),
    };
    Ok(config)
}

fn make_dna_config(path: String) -> Result<DNAConfiguration, String> {
    Ok(DNAConfiguration {
        id: path,
        hash: "DONTCARE".into(),
        file: path,
    })
    // eventually can get actual file content to calculate hash and stuff,
    // but for now it doesn't matter so don't care...

    // let temp = DNAConfiguration {id: "", hash: "", file: dna_path};
    // let dna = Dna::try_from(temp).map_err(|e| e.to_string())?;
}
