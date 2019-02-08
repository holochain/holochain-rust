use cli::{self, package};
use colored::*;
use error::DefaultResult;
use holochain_conductor_api::{
    conductor::{mount_conductor_from_config, CONDUCTOR},
    config::*,
    logger::LogRules,
};
use holochain_core_types::agent::AgentId;
use std::{env, fs};

const LOCAL_STORAGE_PATH: &str = ".hc";

const AGENT_CONFIG_ID: &str = "hc-run-agent";
const DNA_CONFIG_ID: &str = "hc-run-dna";
const INSTANCE_CONFIG_ID: &str = "test-instance";
const INTERFACE_CONFIG_ID: &str = "websocket-interface";

/// Starts a small conductor with the current application running
pub fn run(
    package: bool,
    port: u16,
    persist: bool,
    networked: bool,
    interface: String,
) -> DefaultResult<()> {
    if package {
        cli::package(true, Some(package::DEFAULT_BUNDLE_FILE_NAME.into()))?;
    }

    let agent_name = env::var("HC_AGENT").ok();
    let agent = AgentId::generate_fake(&agent_name.unwrap_or_else(|| String::from("testAgent")));
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

    let interface_type = env::var("HC_INTERFACE").ok().unwrap_or_else(|| interface);
    let driver = if interface_type == String::from("websocket") {
        InterfaceDriver::Websocket { port }
    } else if interface_type == String::from("http") {
        InterfaceDriver::Http { port }
    } else {
        return Err(format_err!("unknown interface type: {}", interface_type));
    };

    let interface_config = InterfaceConfiguration {
        id: INTERFACE_CONFIG_ID.into(),
        driver,
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

    // create an n3h network config if the --networked flag is set
    // or if a value where to find n3h has been put into the
    // HC_N3H_PATH environment variable
    let network_config = if networked || n3h_path.is_some() {
        let n3h_mode = env::var("HC_N3H_MODE").ok();
        let n3h_persistence_path = env::var("HC_N3H_WORK_DIR").ok();
        let n3h_bootstrap_node = env::var("HC_N3H_BOOTSTRAP_NODE").ok();
        let mut n3h_bootstrap = Vec::new();

        if n3h_bootstrap_node.is_some() {
            n3h_bootstrap.push(n3h_bootstrap_node.unwrap())
        }

        // Load end_user config file
        let n3h_end_user_config_filepath = env::var("HC_N3H_END_USER_CONFIG_FILEPATH").ok();

        Some(NetworkConfig {
            bootstrap_nodes: n3h_bootstrap,
            n3h_path: n3h_path.unwrap_or_else(|| default_n3h_path()),
            n3h_mode: n3h_mode.unwrap_or_else(|| default_n3h_mode()),
            n3h_persistence_path: n3h_persistence_path
                .unwrap_or_else(|| default_n3h_persistence_path()),
            n3h_ipc_uri: Default::default(),
            n3h_end_user_config_filepath,
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

    mount_conductor_from_config(base_config);
    let mut conductor_guard = CONDUCTOR.lock().unwrap();
    let conductor = conductor_guard.as_mut().expect("Conductor must be mounted");

    conductor
        .load_config()
        .map_err(|err| format_err!("{}", err))?;

    conductor.start_all_interfaces();
    conductor.start_all_instances()?;

    println!(
        "Holochain development conductor started. Running {} server on port {}",
        interface_type, port
    );
    println!("Type 'exit' to stop the conductor and exit the program");

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

#[cfg(test)]
// flagged as broken for:
// 1. taking 60+ seconds
#[cfg(feature = "broken-tests")]
mod tests {
    use crate::cli::init::{init, tests::gen_dir};
    use assert_cmd::prelude::*;
    use std::{env, process::Command};

    #[test]
    fn test_run() {
        let temp_dir = gen_dir();
        let temp_dir_path = temp_dir.path();
        let temp_dir_path_buf = temp_dir_path.to_path_buf();

        let mut run_cmd = Command::main_binary().unwrap();
        let mut run2_cmd = Command::main_binary().unwrap();

        let _ = init(&temp_dir_path_buf);

        assert!(env::set_current_dir(&temp_dir_path).is_ok());

        let output = run_cmd
            .args(&["run", "--package"])
            .output()
            .expect("should run");
        assert_eq!(format!("{:?}",output),"Output { status: ExitStatus(ExitStatus(256)), stdout: \"\\u{1b}[1;32mCreated\\u{1b}[0m bundle file at \\\"bundle.json\\\"\\nStarting instance \\\"test-instance\\\"...\\nHolochain development conductor started. Running websocket server on port 8888\\nType \\\'exit\\\' to stop the conductor and exit the program\\n\", stderr: \"Error: EOF\\n\" }");

        let output = run2_cmd
            .args(&["run", "--interface", "http"])
            .output()
            .expect("should run");
        assert_eq!(format!("{:?}",output),"Output { status: ExitStatus(ExitStatus(256)), stdout: \"Starting instance \\\"test-instance\\\"...\\nHolochain development conductor started. Running http server on port 8888\\nType \\\'exit\\\' to stop the conductor and exit the program\\n\", stderr: \"Error: EOF\\n\" }");
    }
}
