use holochain_conductor_api::{
    config::{
        AgentConfiguration, Configuration, DnaConfiguration, InstanceConfiguration,
        LoggerConfiguration, StorageConfiguration,
    },
    logger::LogRules,
};
use holochain_core_types::agent::KeyBuffer;
use js_test_conductor::test_key;
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
        // creates a logger which just mutes all
        let mut rules = LogRules::new();
        rules
            .add_rule("^*", true, None)
            .expect("rule is valid");
        LoggerConfiguration {
            logger_type: "debug".into(),
            rules,
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
            let keypair = test_key(&agent_name);
            let pub_key = KeyBuffer::with_corrected(&keypair.get_id()).unwrap();
            let config = AgentConfiguration {
                id: agent_name.clone(),
                name: agent_name.clone(),
                public_address: pub_key.render(),
                key_file: agent_name.clone(),
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
        agents: agent_configs.values().cloned().collect(),
        dnas: dna_configs.values().cloned().collect(),
        instances: instance_configs,
        logger,
        ..Default::default()
    };
    config
}

fn make_dna_config(dna: DnaData) -> Result<DnaConfiguration, String> {
    let path = dna.path.to_string_lossy().to_string();
    Ok(DnaConfiguration {
        id: dna.name.clone(),
        file: path,
        hash: None,
    })
    // eventually can get actual file content to calculate hash and stuff,
    // but for now it doesn't matter so don't care...

    // let temp = DnaConfiguration {id: "", hash: None, file: dna_path};
    // let dna = Dna::try_from(temp).map_err(|e| e.to_string())?;
}
