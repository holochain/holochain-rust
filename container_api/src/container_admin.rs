use crate::{
    config::{DnaConfiguration, InstanceConfiguration},
    container::Container,
};
use holochain_core_types::{cas::content::AddressableContent, error::HolochainError};
use std::{path::PathBuf, sync::Arc};

pub trait ContainerAdmin {
    fn install_dna_from_file(&mut self, path: PathBuf, id: String) -> Result<(), HolochainError>;
    fn uninstall_dna(&mut self, id: String) -> Result<(), HolochainError>;
    fn add_instance_and_start(
        &mut self,
        new_instance: InstanceConfiguration,
    ) -> Result<(), HolochainError>;
    fn remove_instance(&mut self, id: &String) -> Result<(), HolochainError>;
}

impl ContainerAdmin for Container {
    fn install_dna_from_file(&mut self, path: PathBuf, id: String) -> Result<(), HolochainError> {
        let path_string = path
            .to_str()
            .ok_or(HolochainError::ConfigError("invalid path".into()))?;
        let dna =
            Arc::get_mut(&mut self.dna_loader).unwrap()(&path_string.into()).map_err(|e| {
                HolochainError::ConfigError(format!(
                    "Could not load DNA file \"{}\", Error: {}",
                    path_string,
                    e.to_string()
                ))
            })?;

        let new_dna = DnaConfiguration {
            id: id.clone(),
            file: path_string.into(),
            hash: dna.address().to_string(),
        };
        let mut new_config = self.config.clone();
        new_config.dnas.push(new_dna.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        println!("Installed DNA from {} as \"{}\"", path_string, id);
        Ok(())
    }

    fn uninstall_dna(&mut self, _id: String) -> Result<(), HolochainError> {
        Ok(())
    }

    fn add_instance_and_start(
        &mut self,
        instance: InstanceConfiguration,
    ) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        new_config.instances.push(instance.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        self.load_config()?; // populate the instances
        self.start_all_instances() // TODO: create new function to start instance by id to call here
            .map_err(|e| HolochainError::ErrorGeneric(e.to_string()))?;
        println!(
            "Started new instance of {} as \"{}\"",
            instance.dna, instance.id
        );
        Ok(())
    }

    /// Removes the instance given by id from the config.
    /// Also removes all mentions of that instance from all interfaces to not render the config
    /// invalid.
    /// Then saves the config.
    fn remove_instance(&mut self, id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();

        new_config.instances = new_config.instances
            .into_iter()
            .filter(|instance| instance.id != *id)
            .collect();

        new_config.interfaces = new_config.interfaces
            .into_iter()
            .map(|mut interface| {
                interface.instances = interface.instances
                    .into_iter()
                    .filter(|instance| instance.id != *id)
                    .collect();
                interface
            })
            .collect();

        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        println!("Removed instance \"{}\".", id);
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        config::{load_configuration, Configuration},
        container::{tests::example_dna_string, DnaLoader},
    };
    use holochain_core_types::{dna::Dna, json::JsonString};
    use std::{convert::TryFrom, fs::File, io::Read};

    pub fn test_dna_loader() -> DnaLoader {
        let loader =
            Box::new(
                |_: &String| Ok(Dna::try_from(JsonString::from(example_dna_string())).unwrap()),
            ) as Box<FnMut(&String) -> Result<Dna, HolochainError> + Send + Sync>;
        Arc::new(loader)
    }

    pub fn test_toml() -> String {
        r#"bridges = []

[[agents]]
id = "test-agent-1"
key_file = "holo_tester.key"
name = "Holo Tester 1"
public_address = "HoloTester1-----------------------------------------------------------------------AAACZp4xHB"

[[agents]]
id = "test-agent-2"
key_file = "holo_tester.key"
name = "Holo Tester 2"
public_address = "HoloTester2-----------------------------------------------------------------------AAAGy4WW9e"

[[dnas]]
file = "app_spec.hcpkg"
hash = "Qm328wyq38924y"
id = "test-dna"

[[instances]]
agent = "test-agent-1"
dna = "test-dna"
id = "test-instance-1"

[instances.storage]
type = "memory"

[[instances]]
agent = "test-agent-2"
dna = "test-dna"
id = "test-instance-2"

[instances.storage]
type = "memory"

[[interfaces]]
admin = true
id = "websocket interface"

[[interfaces.instances]]
id = "test-instance-1"

[[interfaces.instances]]
id = "test-instance-2"

[interfaces.driver]
port = 3000
type = "websocket"

[logger]
type = ""
[[logger.rules.rules]]
color = "red"
exclude = false
pattern = "^err/"

[[logger.rules.rules]]
color = "white"
exclude = false
pattern = "^debug/dna"

[[logger.rules.rules]]
exclude = false
pattern = ".*"
    "#
            .to_string()
    }

    fn create_test_container() -> Container {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();
        let mut container = Container::from_config(config.clone());
        container.dna_loader = test_dna_loader();
        container.load_config().unwrap();

        let mut tmp_config_path = PathBuf::new();
        tmp_config_path.push("./tmp-test-container-config.toml");
        container.set_config_path(tmp_config_path.clone());
        container
    }

    #[test]
    fn test_install_dna_from_file() {
        let mut container = create_test_container();

        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.hcpkg");

        assert_eq!(
            container.install_dna_from_file(new_dna_path.clone(), String::from("new-dna")),
            Ok(()),
        );

        let new_dna =
            Arc::get_mut(&mut test_dna_loader()).unwrap()(&String::from("new-dna.hcpkg")).unwrap();

        assert_eq!(container.config().dnas.len(), 2,);
        assert_eq!(
            container.config().dnas,
            vec![
                DnaConfiguration {
                    id: String::from("test-dna"),
                    file: String::from("app_spec.hcpkg"),
                    hash: String::from("Qm328wyq38924y"),
                },
                DnaConfiguration {
                    id: String::from("new-dna"),
                    file: String::from("new-dna.hcpkg"),
                    hash: String::from(new_dna.address()),
                },
            ]
        );

        let mut config_contents = String::new();
        let mut file = File::open(&container.config_path).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        assert_eq!(
            config_contents,
r#"bridges = []
interfaces = []

[[agents]]
id = "test-agent-1"
key_file = "holo_tester.key"
name = "Holo Tester 1"
public_address = "HoloTester1-----------------------------------------------------------------------AAACZp4xHB"

[[agents]]
id = "test-agent-2"
key_file = "holo_tester.key"
name = "Holo Tester 2"
public_address = "HoloTester2-----------------------------------------------------------------------AAAGy4WW9e"

[[dnas]]
file = "app_spec.hcpkg"
hash = "Qm328wyq38924y"
id = "test-dna"

[[dnas]]
file = "new-dna.hcpkg"
hash = "QmPB7PJUjwj6zap7jB7oyk616sCRSSnNFRNouqhit6kMTr"
id = "new-dna"

[[instances]]
agent = "test-agent-1"
dna = "test-dna"
id = "test-instance-1"

[instances.storage]
type = "memory"

[[instances]]
agent = "test-agent-2"
dna = "test-dna"
id = "test-instance-2"

[instances.storage]
type = "memory"

[[interfaces]]
admin = true
id = "websocket interface"

[[interfaces.instances]]
id = "test-instance-1"

[[interfaces.instances]]
id = "test-instance-2"

[interfaces.driver]
port = 3000
type = "websocket"

[logger]
type = ""
[[logger.rules.rules]]
color = "red"
exclude = false
pattern = "^err/"

[[logger.rules.rules]]
color = "white"
exclude = false
pattern = "^debug/dna"

[[logger.rules.rules]]
exclude = false
pattern = ".*"
"#
        );
    }

    use crate::config::StorageConfiguration;
    #[test]
    fn test_add_instance_and_start() {
        let mut container = create_test_container();
        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.hcpkg");
        container
            .install_dna_from_file(new_dna_path.clone(), String::from("new-dna"))
            .expect("Could not install DNA");

        let add_result = container.add_instance_and_start(InstanceConfiguration {
            id: String::from("new-instance"),
            dna: String::from("new-dna"),
            agent: String::from("test-agent-1"),
            storage: StorageConfiguration::Memory,
        });

        assert_eq!(add_result, Ok(()))
    }

    #[test]
    /// Tests if the removed instance is gone from the config file
    /// as well as the mentions of the removed instance are gone from the interfaces
    /// (to not render the config invalid).
    fn test_remove_instance() {
        let mut container = create_test_container();
        assert_eq!(
            container.remove_instance(&String::from("test-instance-1")),
            Ok(()),
        );

        let mut config_contents = String::new();
        let mut file = File::open(&container.config_path).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        assert_eq!(
            config_contents,
            r#"bridges = []

[[agents]]
id = "test-agent-1"
key_file = "holo_tester.key"
name = "Holo Tester 1"
public_address = "HoloTester1-----------------------------------------------------------------------AAACZp4xHB"

[[agents]]
id = "test-agent-2"
key_file = "holo_tester.key"
name = "Holo Tester 2"
public_address = "HoloTester2-----------------------------------------------------------------------AAAGy4WW9e"

[[dnas]]
file = "app_spec.hcpkg"
hash = "Qm328wyq38924y"
id = "test-dna"

[[instances]]
agent = "test-agent-2"
dna = "test-dna"
id = "test-instance-2"

[instances.storage]
type = "memory"

[[interfaces]]
admin = true
id = "websocket interface"

[[interfaces.instances]]
id = "test-instance-2"

[interfaces.driver]
port = 3000
type = "websocket"

[logger]
type = ""
[[logger.rules.rules]]
color = "red"
exclude = false
pattern = "^err/"

[[logger.rules.rules]]
color = "white"
exclude = false
pattern = "^debug/dna"

[[logger.rules.rules]]
exclude = false
pattern = ".*"
"#
        );

    }

}
