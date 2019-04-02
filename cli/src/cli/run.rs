use cli;
use colored::*;
use error::DefaultResult;
use holochain_common::env_vars::EnvVar;
use holochain_conductor_api::{
    conductor::{mount_conductor_from_config, CONDUCTOR},
    config::*,
    key_loaders::{test_keystore, test_keystore_loader},
    keystore::PRIMARY_KEYBUNDLE_ID,
    logger::LogRules,
};
use holochain_core_types::agent::AgentId;
use std::{fs, path::PathBuf};

/// Starts a minimal configuration Conductor with the current application running
pub fn run(
    dna_path: PathBuf,
    package: bool,
    port: u16,
    interface_type: String,
    conductor_config: Configuration,
) -> DefaultResult<()> {
    if package {
        cli::package(true, dna_path)?;
    }

    mount_conductor_from_config(conductor_config);
    let mut conductor_guard = CONDUCTOR.lock().unwrap();
    let conductor = conductor_guard.as_mut().expect("Conductor must be mounted");
    conductor.key_loader = test_keystore_loader();

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

pub fn get_interface_type_string(given_type: String) -> String {
    // note that this behaviour is documented within
    // holochain_common::env_vars module and should be updated
    // if this logic changes
    // The environment variable overrides the CLI flag
    EnvVar::Interface.value().ok().unwrap_or_else(|| given_type)
}

pub fn hc_run_configuration(
    dna_path: &PathBuf,
    port: u16,
    persist: bool,
    networked: bool,
    interface_type: &String,
    logging: bool,
) -> DefaultResult<Configuration> {
    Ok(Configuration {
        agents: vec![agent_configuration()],
        dnas: vec![dna_configuration(&dna_path)],
        instances: vec![instance_configuration(storage_configuration(persist)?)],
        interfaces: vec![interface_configuration(&interface_type, port)?],
        network: networking_configuration(networked),
        logger: logger_configuration(logging),
        ..Default::default()
    })
}

// AGENT
const AGENT_NAME_DEFAULT: &str = "testAgent";
const AGENT_CONFIG_ID: &str = "hc-run-agent";

fn agent_configuration() -> AgentConfiguration {
    // note that this behaviour is documented within
    // holochain_common::env_vars module and should be updated
    // if this logic changes
    let agent_name = EnvVar::Agent
        .value()
        .ok()
        .unwrap_or_else(|| String::from(AGENT_NAME_DEFAULT));
    let mut keystore = test_keystore(&agent_name);
    let pub_key = keystore
        .get_keybundle(PRIMARY_KEYBUNDLE_ID)
        .expect("should be able to get keybundle")
        .get_id();
    let agent_id = AgentId::new(&agent_name, pub_key);
    AgentConfiguration {
        id: AGENT_CONFIG_ID.into(),
        name: agent_id.nick,
        public_address: agent_id.pub_sign_key,
        keystore_file: agent_name,
        holo_remote_key: None,
    }
}

// DNA
const DNA_CONFIG_ID: &str = "hc-run-dna";

fn dna_configuration(dna_path: &PathBuf) -> DnaConfiguration {
    DnaConfiguration {
        id: DNA_CONFIG_ID.into(),
        file: dna_path
            .to_str()
            .expect("Expected DNA path to be valid unicode")
            .to_string(),
        hash: None,
    }
}

// STORAGE
const LOCAL_STORAGE_PATH: &str = ".hc";

fn storage_configuration(persist: bool) -> DefaultResult<StorageConfiguration> {
    if persist {
        fs::create_dir_all(LOCAL_STORAGE_PATH)?;

        Ok(StorageConfiguration::Pickle {
            path: LOCAL_STORAGE_PATH.into(),
        })
    } else {
        Ok(StorageConfiguration::Memory)
    }
}

// INSTANCE
const INSTANCE_CONFIG_ID: &str = "test-instance";

fn instance_configuration(storage: StorageConfiguration) -> InstanceConfiguration {
    InstanceConfiguration {
        id: INSTANCE_CONFIG_ID.into(),
        dna: DNA_CONFIG_ID.into(),
        agent: AGENT_CONFIG_ID.into(),
        storage,
    }
}

// INTERFACE
const INTERFACE_CONFIG_ID: &str = "websocket-interface";

fn interface_configuration(
    interface_type: &String,
    port: u16,
) -> DefaultResult<InterfaceConfiguration> {
    let driver = if interface_type == &String::from("websocket") {
        InterfaceDriver::Websocket { port }
    } else if interface_type == &String::from("http") {
        InterfaceDriver::Http { port }
    } else {
        return Err(format_err!("unknown interface type: {}", interface_type));
    };

    Ok(InterfaceConfiguration {
        id: INTERFACE_CONFIG_ID.into(),
        driver,
        admin: true,
        instances: vec![InstanceReferenceConfiguration {
            id: INSTANCE_CONFIG_ID.into(),
        }],
    })
}

// LOGGER
fn logger_configuration(logging: bool) -> LoggerConfiguration {
    // temporary log rules, should come from a configuration
    LoggerConfiguration {
        logger_type: "debug".to_string(),
        rules: if logging {
            LogRules::default()
        } else {
            LogRules::new()
        },
    }
}

// NETWORKING
fn networking_configuration(networked: bool) -> Option<NetworkConfig> {
    // note that this behaviour is documented within
    // holochain_common::env_vars module and should be updated
    // if this logic changes
    let maybe_n3h_path = EnvVar::N3hPath.value().ok();

    // create an n3h network config if the --networked flag is set
    // or if a value where to find n3h has been put into the
    // HC_N3H_PATH environment variable
    if maybe_n3h_path.is_none() && !networked {
        return None;
    }

    // note that this behaviour is documented within
    // holochain_common::env_vars module and should be updated
    // if this logic changes
    let mut bootstrap_nodes = Vec::new();
    if let Ok(node) = EnvVar::N3hBootstrapNode.value() {
        bootstrap_nodes.push(node);
    };

    Some(NetworkConfig {
        bootstrap_nodes,
        n3h_log_level: EnvVar::N3hLogLevel
            .value()
            .ok()
            .unwrap_or_else(default_n3h_log_level),
        n3h_path: maybe_n3h_path.unwrap_or_else(default_n3h_path),
        n3h_mode: EnvVar::N3hMode
            .value()
            .ok()
            .unwrap_or_else(default_n3h_mode),
        n3h_persistence_path: EnvVar::N3hWorkDir
            .value()
            .ok()
            .unwrap_or_else(default_n3h_persistence_path),
        n3h_ipc_uri: Default::default(),
        networking_config_file: EnvVar::NetworkingConfigFile.value().ok(),
    })
}

#[cfg(test)]
mod tests {
    // use crate::cli::init::{init, tests::gen_dir};
    // use assert_cmd::prelude::*;
    // use std::{env, process::Command, path::PathBuf};
    use holochain_conductor_api::config::*;
    use std::path::PathBuf;

    #[test]
    // flagged as broken for:
    // 1. taking 60+ seconds
    // 2. test doesn't take into account dynamic folder for package name
    // 3. test is broken in regard to reading an agent key
    #[cfg(feature = "broken-tests")]
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
        assert_eq!(format!("{:?}",output),"Output { status: ExitStatus(ExitStatus(256)), stdout: \"\\u{1b}[1;32mCreated\\u{1b}[0m dna package file at \\\"x.dna.json\\\"\\nStarting instance \\\"test-instance\\\"...\\nHolochain development conductor started. Running websocket server on port 8888\\nType \\\'exit\\\' to stop the conductor and exit the program\\n\", stderr: \"Error: EOF\\n\" }");

        let output = run2_cmd
            .args(&["run", "--interface", "http"])
            .output()
            .expect("should run");
        assert_eq!(format!("{:?}",output),"Output { status: ExitStatus(ExitStatus(256)), stdout: \"Starting instance \\\"test-instance\\\"...\\nHolochain development conductor started. Running http server on port 8888\\nType \\\'exit\\\' to stop the conductor and exit the program\\n\", stderr: \"Error: EOF\\n\" }");
    }

    #[test]
    fn test_agent_configuration() {
        let agent = super::agent_configuration();
        assert_eq!(
            agent,
            AgentConfiguration {
                id: "hc-run-agent".to_string(),
                name: "testAgent".to_string(),
                public_address: "HcScjN8wBwrn3tuyg89aab3a69xsIgdzmX5P9537BqQZ5A7TEZu7qCY4Xzzjhma"
                    .to_string(),
                keystore_file: "testAgent".to_string(),
                holo_remote_key: None,
            },
        );
    }

    #[test]
    fn test_dna_configuration() {
        let dna_path = PathBuf::from("/test/path");
        let dna = super::dna_configuration(&dna_path);
        assert_eq!(
            dna,
            DnaConfiguration {
                id: "hc-run-dna".to_string(),
                file: "/test/path".to_string(),
                hash: None,
            }
        )
    }

    #[test]
    fn test_storage_configuration() {
        let storage = super::storage_configuration(false).unwrap();
        assert_eq!(storage, StorageConfiguration::Memory);

        let persist_store = super::storage_configuration(true).unwrap();
        assert_eq!(
            persist_store,
            StorageConfiguration::Pickle {
                path: ".hc".to_string()
            }
        );
    }

    #[test]
    fn test_instance_configuration() {
        let storage = super::storage_configuration(false).unwrap();
        let instance = super::instance_configuration(storage);
        assert_eq!(
            instance,
            InstanceConfiguration {
                id: "test-instance".to_string(),
                dna: "hc-run-dna".to_string(),
                agent: "hc-run-agent".to_string(),
                storage: StorageConfiguration::Memory,
            }
        )
    }

    #[test]
    fn test_interface_configuration() {
        let http_interface = super::interface_configuration(&"http".to_string(), 4444).unwrap();
        assert_eq!(
            http_interface,
            InterfaceConfiguration {
                id: "websocket-interface".to_string(),
                driver: InterfaceDriver::Http { port: 4444 },
                admin: true,
                instances: vec![InstanceReferenceConfiguration {
                    id: "test-instance".to_string(),
                }],
            }
        );

        let websocket_interface =
            super::interface_configuration(&"websocket".to_string(), 5555).unwrap();
        assert_eq!(
            websocket_interface,
            InterfaceConfiguration {
                id: "websocket-interface".to_string(),
                driver: InterfaceDriver::Websocket { port: 5555 },
                admin: true,
                instances: vec![InstanceReferenceConfiguration {
                    id: "test-instance".to_string(),
                }],
            }
        );

        let invalid_type = super::interface_configuration(&"funny".to_string(), 4444);
        assert!(invalid_type.is_err());
    }

    #[test]
    fn test_networking_configuration() {
        let networking = super::networking_configuration(true);
        assert_eq!(
            networking,
            Some(NetworkConfig {
                bootstrap_nodes: Vec::new(),
                n3h_log_level: default_n3h_log_level(),
                n3h_path: default_n3h_path(),
                n3h_mode: default_n3h_mode(),
                n3h_persistence_path: default_n3h_persistence_path(),
                n3h_ipc_uri: Default::default(),
                networking_config_file: None,
            })
        );

        let no_networking = super::networking_configuration(false);
        assert!(no_networking.is_none());
    }
}
