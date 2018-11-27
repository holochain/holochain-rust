use cli::{self, package};
use error::DefaultResult;
use holochain_container_api::{
    config::{AgentConfiguration, Configuration, DNAConfiguration},
    container::Container,
};

pub fn run(package: bool) -> DefaultResult<()> {
    if package {
        cli::package(true, Some(package::DEFAULT_BUNDLE_FILE_NAME.into()))?;
    }

    let agent_config = AgentConfiguration {
        id: "hc-run".into(),
        key_file: "hc_run.key".into(),
    };

    let dna_config = DNAConfiguration {
        id: "dna.local".into(),
        file: package::DEFAULT_BUNDLE_FILE_NAME.into(),
        hash: "Qm328wyq38924ybogus".into(),
    };

    let base_config = Configuration {
        agents: vec![agent_config],
        dnas: vec![dna_config],
        ..Default::default()
    };

    let mut container = Container::with_config(base_config);

    container.start_all_interfaces();
    container.start_all_instances()?;

    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ")?;

        match readline.as_str() {
            "exit" => break,
            _ => {}
        }
    }

    Ok(())
}
