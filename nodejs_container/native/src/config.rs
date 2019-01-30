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
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceData {
    pub agent: AgentData,
    pub dna: DnaData,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MakeConfigOptions {
    #[serde(default, rename = "debugLog")]
    debug_log: bool,
}

pub fn js_make_config(mut cx: FunctionContext) -> JsResult<JsValue> {
    let instances_arg: Handle<JsValue> = cx.argument(0)?;
    let opts_arg: Handle<JsValue> = cx.argument(1)?;

    let instances: Vec<InstanceData> = neon_serde::from_value(&mut cx, instances_arg)?;
    let opts: MakeConfigOptions = neon_serde::from_value(&mut cx, opts_arg)?;
    let logger = if opts.debug_log {
        Default::default()
    } else {
        LoggerConfiguration {
            logger_type: "debug".into(),
            rules: LogRules::new(),
        }
    };
    let config = make_config(instances, logger);
    Ok(neon_serde::to_value(&mut cx, &config)?)
}

fn make_config(instance_data: Vec<InstanceData>, logger: LoggerConfiguration) -> Configuration {
    let mut agent_configs = HashMap::new();
    let mut dna_configs = HashMap::new();
    let mut instance_configs = Vec::new();
    for instance in instance_data {
        let agent_name = instance.agent.name;
        let mut dna_data = instance.dna;
        let agent_config = agent_configs.entry(agent_name.clone()).or_insert_with(|| {
            let agent_key = AgentId::generate_fake(&agent_name);
            let config = AgentConfiguration {
                id: agent_name.clone(),
                name: agent_name.clone(),
                public_address: agent_key.key,
                key_file: format!("fake/key/{}", agent_name),
            };
            config
        });
        let dna_config = dna_configs
            .entry(dna_data.path.clone())
            .or_insert_with(|| make_dna_config(dna_data).expect("DNA file not found"));

        let agent_id = agent_config.id.clone();
        let dna_id = dna_config.id.clone();
        let instance = InstanceConfiguration {
            id: instance.name,
            agent: agent_id,
            dna: dna_id,
            storage: StorageConfiguration::Memory,
        };
        instance_configs.push(instance);
    }

    let config = Configuration {
        agents: agent_configs.into_iter().map(|(_, v)| v).collect(),
        dnas: dna_configs.into_iter().map(|(_, v)| v).collect(),
        instances: instance_configs,
        logger,
        ..Default::default()
    };
    config
}

fn instance_id(agent_id: &str, dna_id: &str) -> String {
    format!("{}::{}", agent_id, dna_id)
}

pub fn js_instance_id(mut cx: FunctionContext) -> JsResult<JsString> {
    let agent_id = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
    let dna_id = cx.argument::<JsString>(1)?.to_string(&mut cx)?.value();
    let id = instance_id(&agent_id, &dna_id);
    Ok(cx.string(id))
}

fn make_dna_config(dna: DnaData) -> Result<DnaConfiguration, String> {
    let path = dna.path.to_string_lossy().to_string();
    Ok(DnaConfiguration {
        id: dna.name.clone(),
        hash: String::from("DONTCARE"),
        file: path,
    })
    // eventually can get actual file content to calculate hash and stuff,
    // but for now it doesn't matter so don't care...

    // let temp = DnaConfiguration {id: "", hash: "", file: dna_path};
    // let dna = Dna::try_from(temp).map_err(|e| e.to_string())?;
}
