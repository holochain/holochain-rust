use cli::{self, package};
use error::DefaultResult;
use holochain_container_api::{config::*, container::Container};
use notify::{watcher, RecursiveMode, Watcher};
use std::{sync::mpsc, time::Duration};

/// Starts a small container with the current application running
pub fn run(package: bool, port: u16, watch: bool) -> DefaultResult<()> {
    if package {
        cli::package(true, Some(package::DEFAULT_BUNDLE_FILE_NAME.into()))?;
    }

    let agent_config = AgentConfiguration {
        id: "hc-run-agent".into(),
        key_file: "hc_run.key".into(),
    };

    let dna_config = DNAConfiguration {
        id: "hc-run-dna".into(),
        file: package::DEFAULT_BUNDLE_FILE_NAME.into(),
        hash: "Qm328wyq38924ybogus".into(),
    };

    let instance_config = InstanceConfiguration {
        id: "test-instance".into(),
        dna: "hc-run-dna".into(),
        agent: "hc-run-agent".into(),
        logger: Default::default(),
        storage: StorageConfiguration::Memory,
    };

    let interface_config = InterfaceConfiguration {
        id: "websocket-interface".into(),
        driver: InterfaceDriver::Websocket { port: port },
        admin: true,
        instances: vec![InstanceReferenceConfiguration {
            id: "test-instance".into(),
        }],
    };

    let base_config = Configuration {
        agents: vec![agent_config],
        dnas: vec![dna_config],
        instances: vec![instance_config],
        interfaces: vec![interface_config],
        ..Default::default()
    };

    let mut container = Container::with_config(base_config.clone());

    container
        .load_config(&base_config)
        .map_err(|err| format_err!("{}", err))?;

    container.start_all_interfaces();
    container.start_all_instances()?;

    println!(
        "Holochain development container started. Running websocket server on port {}",
        port
    );

    if watch {
        println!("hc is watching files watching files...");
        println!("Note: The interactive shell has been deactivated for that purpose");

        let (tx, rx) = mpsc::channel();

        let mut watcher = watcher(tx, Duration::from_secs(1))?;

        watcher.watch(".", RecursiveMode::Recursive)?;

        loop {
            while rx.recv().is_ok() {
                println!("Files have changed. Rebuilding...");

                cli::package(true, Some(package::DEFAULT_BUNDLE_FILE_NAME.into()))?;
            }
        }
    } else {
        println!("Type 'exit' to stop the container and exit the program");

        let mut rl = rustyline::Editor::<()>::new();

        loop {
            let readline = rl.readline("hc> ")?;

            match readline.as_str() {
                "exit" => break,
                other if !other.is_empty() => eprintln!(
                    "command {:?} not recognized. Available commands are: exit",
                    other
                ),
                _ => continue,
            }
        }
    }

    Ok(())
}
