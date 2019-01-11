use holochain_container_api::{
    config::{
        AgentConfiguration, Configuration, DnaConfiguration, InstanceConfiguration,
        LoggerConfiguration, StorageConfiguration,
    },
    logger::LogRules,
};
use holochain_core_types::agent::AgentId;
use neon::prelude::*;
use std::{collections::HashMap, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentData {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DnaData {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceData {
    pub agent: AgentData,
    pub dna: DnaData,
}

pub fn js_make_config(mut cx: FunctionContext) -> JsResult<JsValue> {
    let instances_arg: Handle<JsValue> = cx.argument(0)?;
    let instances: Vec<InstanceData> = neon_serde::from_value(&mut cx, instances_arg)?;
    let config = make_config(instances);
    Ok(neon_serde::to_value(&mut cx, &config)?)
}

fn make_config(instance_data: Vec<InstanceData>) -> Configuration {
    let mut agent_configs = HashMap::new();
    let mut dna_configs = HashMap::new();
    let mut instance_configs = Vec::new();
    for instance in instance_data {
        let agent_name = instance.agent.name;
        let mut dna_data = instance.dna;
        let agent_config = agent_configs.entry(agent_name.clone()).or_insert_with(|| {
            let agent_key = AgentId::generate_fake(&agent_name);
            AgentConfiguration {
                id: agent_name.clone(),
                name: agent_name.clone(),
                public_address: agent_key.key,
                key_file: format!("fake/key/{}", agent_name),
            }
        });
        let dna_config = dna_configs
            .entry(dna_data.path.clone())
            .or_insert_with(|| make_dna_config(dna_data).expect("DNA file not found"));

        let agent_id = agent_config.id.clone();
        let dna_id = dna_config.id.clone();
        let instance = InstanceConfiguration {
            id: instance_id(&agent_id, &dna_id),
            agent: agent_id,
            dna: dna_id,
            storage: StorageConfiguration::Memory,
        };
        instance_configs.push(instance);
    }

    let config = Configuration {
        agents: agent_configs.values().cloned().collect(),
        dnas: dna_configs.values().cloned().collect(),
        instances: instance_configs,
        interfaces: Vec::new(),
        bridges: Vec::new(),
        logger: LoggerConfiguration {
            logger_type: "debug".to_string(),
            rules: LogRules::new(),
        },
        ..Default::default()
    };
    config
}

fn instance_id(agent_id: &str, dna_id: &str) -> String {
    format!("{}::{}", agent_id, dna_id)
}

fn make_dna_config(dna: DnaData) -> Result<DnaConfiguration, String> {
    let path = dna.path.to_string_lossy().to_string();
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
