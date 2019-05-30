use holochain_conductor_api::{
    config::{
        AgentConfiguration, Bridge, Configuration, DnaConfiguration, DpkiConfiguration,
        InstanceConfiguration, LoggerConfiguration, SignalConfig, StorageConfiguration,
    },
    key_loaders::test_keystore,
    keystore::PRIMARY_KEYBUNDLE_ID,
    logger::LogRules,
};
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

#[derive(Serialize, Deserialize, Debug, Default)]
struct MakeConfigOptions {
    #[serde(default)]
    instances: Vec<InstanceData>,

    #[serde(default)]
    bridges: Vec<Bridge>,

    #[serde(default)]
    dpki: Option<DpkiConfiguration>,

    #[serde(default, rename = "debugLog")]
    debug_log: bool,
}

pub fn js_make_config(mut cx: FunctionContext) -> JsResult<JsValue> {
    let opts: MakeConfigOptions = {
        let first: Handle<JsValue> = cx.argument(0)?;
        // This condition is introduced to allow all config data to be passed in a single object
        // while maintaing backwards compatibility with the method of supplying the instance list
        // separately as a first argument.
        if first.is_a::<JsArray>() {
            // Read instance Array from first argument
            let instances: Vec<InstanceData> = neon_serde::from_value(&mut cx, first)?;
            // Read extra options from second argument, or return empty object if non-existant
            let mut opts = cx
                .argument(1)
                .and_then(|second| {
                    let opts: MakeConfigOptions = neon_serde::from_value(&mut cx, second)?;
                    Ok(opts)
                })
                .unwrap_or(MakeConfigOptions::default());
            // Overwrite instances in options with value from first argument
            opts.instances = instances;
            opts
        } else {
            let opts: MakeConfigOptions = neon_serde::from_value(&mut cx, first)?;
            if opts.instances.is_empty() {
                return cx.throw_error("`instances` cannot be empty");
            }
            opts
        }
    };

    let logger = if opts.debug_log {
        Default::default()
    } else {
        // creates a logger which just mutes all
        let mut rules = LogRules::new();
        rules.add_rule("^*", true, None).expect("rule is valid");
        LoggerConfiguration {
            logger_type: "debug".into(),
            rules,
        }
    };
    let config = make_config(opts.instances, opts.bridges, opts.dpki, logger);
    Ok(neon_serde::to_value(&mut cx, &config)?)
}

fn make_config(
    instance_data: Vec<InstanceData>,
    bridges: Vec<Bridge>,
    dpki: Option<DpkiConfiguration>,
    logger: LoggerConfiguration,
) -> Configuration {
    let mut agent_configs = HashMap::new();
    let mut dna_configs = HashMap::new();
    let mut instance_configs = Vec::new();
    for instance in instance_data {
        let agent_name = instance.agent.name;
        let mut dna_data = instance.dna;
        let agent_config = agent_configs.entry(agent_name.clone()).or_insert_with(|| {
            let mut keystore = test_keystore(&agent_name);
            let keybundle = keystore
                .get_keybundle(PRIMARY_KEYBUNDLE_ID)
                .expect("Couldn't get KeyBundle that was just added back from Keystore");
            let config = AgentConfiguration {
                id: agent_name.clone(),
                name: agent_name.clone(),
                public_address: keybundle.get_id(),
                keystore_file: agent_name.clone(),
                holo_remote_key: None,
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

    let signals = SignalConfig {
        trace: true,
        consistency: false,
    };

    let config = Configuration {
        agents: agent_configs.values().cloned().collect(),
        dnas: dna_configs.values().cloned().collect(),
        instances: instance_configs,
        bridges,
        dpki,
        logger,
        signals,
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
