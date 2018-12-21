use holochain_container_api::config::{
    AgentConfiguration, Configuration, DnaConfiguration, InstanceConfiguration,
    LoggerConfiguration, StorageConfiguration,
};
use holochain_core_types::agent::AgentId;
use holochain_net::p2p_config::P2pConfig;
use neon::prelude::*;
use std::{collections::HashMap, path::PathBuf};

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

/// Provides functions for create the building blocks of configuration.
/// These pieces can be assembled together to create different configurations
/// which can be used to create Habitats
pub struct ConfigBuilder;

declare_types! {

    pub class JsConfigBuilder for ConfigBuilder {

        init(_cx) {
            Ok(ConfigBuilder)
        }

        method agent(mut cx) {
            let name = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let obj = AgentData { name };
            Ok(neon_serde::to_value(&mut cx, &obj)?)
        }

        method dna(mut cx) {
            let path = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let path = PathBuf::from(path);
            let obj = DnaData { path };
            Ok(neon_serde::to_value(&mut cx, &obj)?)
        }

        method instance(mut cx) {
            let agent_data = cx.argument(0)?;
            let dna_data = cx.argument(1)?;
            let agent: AgentData = neon_serde::from_value(&mut cx, agent_data)?;
            let dna: DnaData = neon_serde::from_value(&mut cx, dna_data)?;
            let obj = InstanceData { agent, dna };
            Ok(neon_serde::to_value(&mut cx, &obj)?)
        }

        method container(mut cx) {
            let mut i = 0;
            let mut instances = Vec::<InstanceData>::new();
            while let Some(arg) = cx.argument_opt(i) {
                instances.push(neon_serde::from_value(&mut cx, arg)?);
                i += 1;
            };
            let config = make_config(instances);
            Ok(neon_serde::to_value(&mut cx, &config)?)
        }
    }
}

pub fn make_config(instance_data: Vec<InstanceData>) -> Configuration {
    let mut agent_configs = HashMap::new();
    let mut dna_configs = HashMap::new();
    let mut instance_configs = Vec::new();
    for instance in instance_data {
        let agent_name = instance.agent.name;
        let dna_path = PathBuf::from(instance.dna.path);
        let agent = agent_configs.entry(agent_name.clone()).or_insert_with(|| {
            let agent_key = AgentId::generate_fake(&agent_name);
            AgentConfiguration {
                id: agent_name.clone(),
                name: agent_name.clone(),
                public_address: agent_key.key,
                key_file: format!("fake/key/{}", agent_name),
            }
        });
        let dna = dna_configs
            .entry(dna_path.clone())
            .or_insert_with(|| make_dna_config(dna_path).expect("DNA file not found"));

        let logger_mock = LoggerConfiguration {
            logger_type: String::from("DONTCARE"),
            file: None,
        };
        let network_mock = Some(P2pConfig::DEFAULT_MOCK_CONFIG.to_string());
        let agent_id = agent.id.clone();
        let dna_id = dna.id.clone();
        let instance = InstanceConfiguration {
            id: instance_id(&agent_id, &dna_id),
            agent: agent_id,
            dna: dna_id,
            storage: StorageConfiguration::Memory,
            logger: logger_mock,
            network: network_mock,
        };
        instance_configs.push(instance);
    }

    let config = Configuration {
        agents: agent_configs.into_iter().map(|(_, v)| v).collect(),
        dnas: dna_configs.into_iter().map(|(_, v)| v).collect(),
        instances: instance_configs,
        interfaces: Vec::new(),
        bridges: Vec::new(),
    };
    config
}

fn instance_id(agent_id: &str, dna_id: &str) -> String {
    format!("{}-{}", agent_id, dna_id)
}

fn make_dna_config(path: PathBuf) -> Result<DnaConfiguration, String> {
    let path = path.to_string_lossy().to_string();
    Ok(DnaConfiguration {
        id: path.clone(),
        hash: String::from("DONTCARE"),
        file: path,
    })
    // eventually can get actual file content to calculate hash and stuff,
    // but for now it doesn't matter so don't care...

    // let temp = DnaConfiguration {id: "", hash: "", file: dna_path};
    // let dna = Dna::try_from(temp).map_err(|e| e.to_string())?;
}
