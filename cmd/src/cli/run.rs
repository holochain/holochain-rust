use cli::{self, package};
use colored::*;
use error::DefaultResult;
use holochain_container_api::{config::*, container::Container, logger::LogRules};
use holochain_core_types::agent::AgentId;
use std::{env, fs};

const LOCAL_STORAGE_PATH: &str = ".hc";

const AGENT_CONFIG_ID: &str = "hc-run-agent";
const DNA_CONFIG_ID: &str = "hc-run-dna";
const INSTANCE_CONFIG_ID: &str = "test-instance";
const INTERFACE_CONFIG_ID: &str = "websocket-interface";

/// Starts a small container with the current application running
pub fn run(package: bool, port: u16, persist: bool, networked: bool) -> DefaultResult<()> {
    if package {
        cli::package(true, Some(package::DEFAULT_BUNDLE_FILE_NAME.into()))?;
    }

    let agent = AgentId::generate_fake("testAgent");
    let agent_config = AgentConfiguration {
        id: AGENT_CONFIG_ID.into(),
        name: agent.nick,
        public_address: agent.key,
        key_file: "hc_run.key".into(),
    };

    let dna_config = DnaConfiguration {
        id: DNA_CONFIG_ID.into(),
        file: package::DEFAULT_BUNDLE_FILE_NAME.into(),
        hash: "Qm328wyq38924ybogus".into(),
    };

    let storage = if persist {
        fs::create_dir_all(LOCAL_STORAGE_PATH)?;

        StorageConfiguration::File {
            path: LOCAL_STORAGE_PATH.into(),
        }
    } else {
        StorageConfiguration::Memory
    };

    let instance_config = InstanceConfiguration {
        id: INSTANCE_CONFIG_ID.into(),
        dna: DNA_CONFIG_ID.into(),
        agent: AGENT_CONFIG_ID.into(),
        storage,
    };

    let interface_config = InterfaceConfiguration {
        id: INTERFACE_CONFIG_ID.into(),
        driver: InterfaceDriver::Websocket { port },
        admin: true,
        instances: vec![InstanceReferenceConfiguration {
            id: INSTANCE_CONFIG_ID.into(),
        }],
    };

    // temporary log rules, should come from a configuration
    let rules = LogRules::new();
    let logger_config = LoggerConfiguration {
        logger_type: "debug".to_string(),
        rules,
    };

    let n3h_path = env::var("HC_N3H_PATH").ok();
    let n3h_mode = env::var("HC_N3H_MODE").ok();

    // network config
    let network_config = if networked || n3h_path.is_some() {
        Some(NetworkConfig {
            bootstrap_nodes: Default::default(),
            n3h_path: n3h_path.unwrap_or_else(|| default_n3h_path()),
            n3h_mode: n3h_mode.unwrap_or_else(|| default_n3h_mode()),
            n3h_persistence_path: Default::default(),
            n3h_ipc_uri: Default::default(),
        })
    } else {
        None
    };

    let base_config = Configuration {
        agents: vec![agent_config],
        dnas: vec![dna_config],
        instances: vec![instance_config],
        interfaces: vec![interface_config],
        network: network_config,
        logger: logger_config,
        ..Default::default()
    };

    let mut container = Container::from_config(base_config.clone());

    container
        .load_config()
        .map_err(|err| format_err!("{}", err))?;

    container.start_all_interfaces();
    container.start_all_instances()?;

    println!(
        "Holochain development container started. Running websocket server on port {}",
        port
    );
    println!("Type 'exit' to stop the container and exit the program");

    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let readline = rl.readline("hc> ")?;

        match readline.as_str().trim() {
            "exit" => break,
            other if !other.is_empty() => eprintln!(
                "command {} not recognized. Available commands are: exit",
                other.red().bold()
            ),
            _ => continue,
        }
    }

    Ok(())
}
