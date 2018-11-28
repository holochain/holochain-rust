use cli::{self, package};
use error::DefaultResult;
use holochain_container_api::{
    config::*,
    container::Container,
};

/// Starts a small container with the current application running
pub fn run(package: bool) -> DefaultResult<()> {
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
        id: "hc-run-instance".into(),
        dna: "hc-run-dna".into(),
        agent: "hc-run-agent".into(),
        logger: Default::default(),
        storage: StorageConfiguration::Memory,
    };

    let interface_config = InterfaceConfiguration {
        id: "websocket-interface".into(),
        driver: InterfaceDriver::Websocket { port: 8888 },
        admin: true,
        instances: vec![InstanceReferenceConfiguration {
            id: "hc-run-instance".into()
        }],
    };

    let base_config = Configuration {
        agents: vec![agent_config],
        dnas: vec![dna_config],
        instances: vec![instance_config],
        interfaces: vec![interface_config],
        ..Default::default()
    };

    let mut container = Container::with_config(base_config);

    container.start_all_interfaces();
    container.start_all_instances()?;

    println!("Holochain development container started!");
    println!("Type 'exit' to stop the container and exit the program");
    println!();

    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ")?;

        match readline.as_str() {
            "exit" => break,
            _ => println!("command {:?} not recognized. Available commands are: exit", readline)
        }
    }

    Ok(())
}
