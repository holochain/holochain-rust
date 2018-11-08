use Holochain;
use config::Configuration;
use std::collections::HashMap;
use holochain_dna::Dna;
use holochain_cas_implementations::{
    cas::file::FilesystemStorage, eav::file::EavFileStorage, path::create_path_if_not_exists,
};
use holochain_core::context::Context;
use holochain_core_types::error::HolochainError;
use std::sync::Arc;

use holochain_core::{logger::Logger, persister::SimplePersister};
use holochain_core_types::entry::agent::Agent;
use std::{
    sync::Mutex,
};

use boolinator::*;

pub struct Container {
    instances: HashMap<String, Holochain>,
}

impl Container {
    pub fn shutdown(mut self) {
        self.instances = self.instances
            .into_iter()
            .map(|(id, mut hc)| {
                let _ = hc.stop();
                (id,hc)
            })
            .collect::<HashMap<_,_>>();
    }

    pub fn load_config(self, config: &Configuration) -> Result<(), String> {
        let _ = config.check_consistency()?;
        self.shutdown();



        Ok(())
    }


}

type DnaLoader = Box<FnMut(&String) -> Result<Dna, String> + Send>;

fn instantiate_from_config(id: &String, config: &Configuration, mut dna_loader: DnaLoader) -> Result<Holochain, String>{
    let _ = config.check_consistency()?;

    config.instance_by_id(&id)
        .ok_or(String::from("Instance not found in config"))
        .and_then(|instance_config| {
            let agent_config = config.agent_by_id(&instance_config.agent).unwrap();
            let dna_config = config.dna_by_id(&instance_config.dna).unwrap();
            let dna = dna_loader((&dna_config.file))?;

            (instance_config.storage.storage_type == "file" && instance_config.storage.path.is_some())
                .ok_or(String::from("Only file storage supported currently"))?;

            let context = create_context(&agent_config.id, &instance_config.storage.path.unwrap())
                .map_err(|hc_err| format!("Error creating context: {}", hc_err.to_string()))?;

            Ok(
                Holochain::new(dna, Arc::new(context))
                .map_err(|hc_err| hc_err.to_string())?
            )
        })
}

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

fn create_context(agent: &String, path: &String) -> Result<Context, HolochainError> {
    let agent = Agent::from("c_bob".to_string());
    let cas_path = format!("{}/cas", path);
    let eav_path = format!("{}/eav", path);
    let agent_path = format!("{}/state", path);
    create_path_if_not_exists(&cas_path)?;
    create_path_if_not_exists(&eav_path)?;
    Context::new(
        agent,
        Arc::new(Mutex::new(NullLogger {})),
        Arc::new(Mutex::new(SimplePersister::new(agent_path))),
        FilesystemStorage::new(&cas_path)?,
        EavFileStorage::new(eav_path)?,
    )
}

mod tests {
    use super::*;
    use config::load_configuration;

    fn test_dna(file: &String) -> Result<Dna, String> {
        Ok(Dna::new())
    }

    #[test]
    fn test_instantiate_from_config() {
        let toml = r#"
    [[agents]]
    id = "test agent"
    name = "Holo Tester"
    key_file = "holo_tester.key"

    [[dnas]]
    id = "app spec rust"
    file = "app-spec-rust.hcpkg"
    hash = "Qm328wyq38924y"

    [[instances]]
    id = "app spec instance"
    dna = "app spec rust"
    agent = "test agent"
    [instances.logger]
    type = "simple"
    file = "app_spec.log"
    [instances.storage]
    type = "file"
    path = "."

    "#;
        let config = load_configuration::<Configuration>(toml).unwrap();
        let maybe_holochain = instantiate_from_config(&"app spec instance".to_string(), &config, Box::new(test_dna));

        assert_eq!(maybe_holochain.err(), None);

    }
}