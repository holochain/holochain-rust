use cli::{self, package};
use colored::*;
use error::DefaultResult;
use holochain_container_api::{config::*, container::Container};
use holochain_core_types::agent::AgentId;
use holochain_net::p2p_config::P2pConfig;
use std::fs;

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
        logger: Default::default(),
        storage,
        network: Some(if networked {
            P2pConfig::default_ipc().as_str()
        } else {
            P2pConfig::default_mock().as_str()
        }),
    };

    let interface_config = InterfaceConfiguration {
        id: INTERFACE_CONFIG_ID.into(),
        driver: InterfaceDriver::Websocket { port },
        admin: true,
        instances: vec![InstanceReferenceConfiguration {
            id: INSTANCE_CONFIG_ID.into(),
        }],
    };

    let base_config = Configuration {
        agents: vec![agent_config],
        dnas: vec![dna_config],
        instances: vec![instance_config],
        interfaces: vec![interface_config],
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
