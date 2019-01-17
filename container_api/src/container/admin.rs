use crate::{
    config::{DnaConfiguration, InstanceConfiguration},
    container::{base::notify, Container},
    error::HolochainInstanceError,
};
use holochain_core_types::{cas::content::AddressableContent, error::HolochainError};
use json_patch;
use std::{path::PathBuf, sync::Arc};

pub trait ContainerAdmin {
    fn install_dna_from_file(
        &mut self,
        path: PathBuf,
        id: String,
        copy: bool,
        properties: Option<&serde_json::Value>,
    ) -> Result<(), HolochainError>;
    fn uninstall_dna(&mut self, id: &String) -> Result<(), HolochainError>;
    fn add_instance(&mut self, new_instance: InstanceConfiguration) -> Result<(), HolochainError>;
    fn remove_instance(&mut self, id: &String) -> Result<(), HolochainError>;
    fn start_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
    fn stop_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
}

impl ContainerAdmin for Container {
    fn install_dna_from_file(
        &mut self,
        path: PathBuf,
        id: String,
        copy: bool,
        properties: Option<&serde_json::Value>,
    ) -> Result<(), HolochainError> {
        let path_string = path
            .to_str()
            .ok_or(HolochainError::ConfigError("invalid path".into()))?;
        let mut dna =
            Arc::get_mut(&mut self.dna_loader).unwrap()(&path_string.into()).map_err(|e| {
                HolochainError::ConfigError(format!(
                    "Could not load DNA file \"{}\", Error: {}",
                    path_string,
                    e.to_string()
                ))
            })?;

        if let Some(props) = properties {
            if !copy {
                return Err(HolochainError::ConfigError(
                    "Cannot install DNA with properties unless copy flag is true".into(),
                ));
            }
            json_patch::merge(&mut dna.properties, &props);
        }

        let config_path = match copy {
            true => self.save_dna(&dna)?,
            false => PathBuf::from(path_string),
        };
        let config_path_str = config_path
            .to_str()
            .ok_or(HolochainError::ConfigError("invalid path".into()))?;

        let new_dna = DnaConfiguration {
            id: id.clone(),
            file: config_path_str.into(),
            hash: dna.address().to_string(),
        };

        let mut new_config = self.config.clone();
        new_config.dnas.push(new_dna.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        notify(format!("Installed DNA from {} as \"{}\"", path_string, id));
        Ok(())
    }

    /// Removes the DNA given by id from the config.
    /// Also removes all instances and their mentions from all interfaces to not render the config
    /// invalid.
    /// Then saves the config.
    fn uninstall_dna(&mut self, id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        new_config.dnas = new_config
            .dnas
            .into_iter()
            .filter(|dna| dna.id != *id)
            .collect();

        let instance_ids: Vec<String> = new_config
            .instances
            .iter()
            .filter(|instance| instance.dna == *id)
            .map(|instance| instance.id.clone())
            .collect();

        for id in instance_ids.iter() {
            new_config = new_config.save_remove_instance(id);
        }

        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        for id in instance_ids.iter() {
            let result = self.stop_instance(id);
            if result.is_err() {
                notify(format!(
                    "Error stopping instance {}: \"{}\".",
                    id,
                    result.err().unwrap()
                ));
            }
            notify(format!("Removed instance \"{}\".", id));
        }

        notify(format!("Uninstalled DNA \"{}\".", id));

        Ok(())
    }

    fn add_instance(&mut self, instance: InstanceConfiguration) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        new_config.instances.push(instance.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        Ok(())
    }

    /// Removes the instance given by id from the config.
    /// Also removes all mentions of that instance from all interfaces to not render the config
    /// invalid.
    /// Then saves the config.
    fn remove_instance(&mut self, id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();

        new_config = new_config.save_remove_instance(id);

        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        let result = self.stop_instance(id);
        if result.is_err() {
            notify(format!(
                "Error stopping instance {}: \"{}\".",
                id,
                result.err().unwrap()
            ));
        }
        self.instances.remove(id);

        notify(format!("Removed instance \"{}\".", id));
        Ok(())
    }

    fn start_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError> {
        let instance = self.instances.get(id)?;
        notify(format!("Starting instance \"{}\"...", id));
        instance.write().unwrap().start()
    }

    fn stop_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError> {
        let instance = self.instances.get(id)?;
        notify(format!("Stopping instance \"{}\"...", id));
        instance.write().unwrap().stop()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        config::{load_configuration, Configuration},
        container::base::{tests::example_dna_string, DnaLoader},
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
persistence_dir = "./tmp-test/"

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

    fn create_test_container<T: Into<String>>(test_name: T) -> Container {
        let config = load_configuration::<Configuration>(&test_toml()).unwrap();
        let mut container = Container::from_config(config.clone());
        container.dna_loader = test_dna_loader();
        container.load_config().unwrap();

        let mut tmp_config_path = PathBuf::new();
        tmp_config_path.push(format!("./tmp-{}-container-config.toml", test_name.into()));
        container.set_config_path(tmp_config_path.clone());
        container
    }

    #[test]
    fn test_install_dna_from_file() {
        let mut container = create_test_container("test_install_dna_from_file");

        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.hcpkg");

        assert_eq!(
            container.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna"),
                false,
                None
            ),
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
persistence_dir = "./tmp-test/"

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

    #[test]
    fn test_install_dna_from_file_and_copy() {
        let mut container = create_test_container("test_install_dna_from_file_and_copy");

        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.hcpkg");

        assert_eq!(
            container.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna"),
                true,
                None
            ),
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
                    file: format!("./tmp-test/dna/{}.hcpkg", new_dna.address()),
                    hash: String::from(new_dna.address()),
                },
            ]
        );
        assert!(PathBuf::from(format!("./tmp-test/dna/{}.hcpkg", new_dna.address())).is_file())
    }

    #[test]
    fn test_install_dna_from_file_with_properties() {
        let mut container = create_test_container("test_install_dna_from_file_with_properties");

        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.hcpkg");
        let new_props = json!({"propertyKey": "value"});

        assert_eq!(
            container.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna-with-props"),
                false,
                Some(&new_props)
            ),
            Err(HolochainError::ConfigError("Cannot install DNA with properties unless copy flag is true".into())),
        );

        assert_eq!(
            container.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna-with-props"),
                true,
                Some(&new_props)
            ),
            Ok(()),
        );

        let mut new_dna =
            Arc::get_mut(&mut test_dna_loader()).unwrap()(&String::from("new-dna.hcpkg")).unwrap();
        let original_hash = new_dna.address();
        new_dna.properties = new_props;
        let new_hash = new_dna.address();
        assert_ne!(original_hash, new_hash);
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
                    id: String::from("new-dna-with-props"),
                    file: format!("./tmp-test/dna/{}.hcpkg", new_dna.address()),
                    hash: String::from(new_dna.address()),
                },
            ]
        );
        assert!(PathBuf::from(format!("./tmp-test/dna/{}.hcpkg", new_dna.address())).is_file())
    }

    use crate::config::StorageConfiguration;
    #[test]
    fn test_add_instance() {
        let mut container = create_test_container("test_add_instance");
        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.hcpkg");
        container
            .install_dna_from_file(new_dna_path.clone(), String::from("new-dna"), false, None)
            .expect("Could not install DNA");

        let add_result = container.add_instance(InstanceConfiguration {
            id: String::from("new-instance"),
            dna: String::from("new-dna"),
            agent: String::from("test-agent-1"),
            storage: StorageConfiguration::Memory,
        });

        assert_eq!(add_result, Ok(()));

        let mut config_contents = String::new();
        let mut file = File::open(&container.config_path).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        assert_eq!(
            config_contents,
       r#"bridges = []
persistence_dir = "./tmp-test/"

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

[[instances]]
agent = "test-agent-1"
dna = "new-dna"
id = "new-instance"

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

    #[test]
    /// Tests if the removed instance is gone from the config file
    /// as well as the mentions of the removed instance are gone from the interfaces
    /// (to not render the config invalid).
    fn test_remove_instance() {
        let mut container = create_test_container("test_remove_instance");
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
persistence_dir = "./tmp-test/"

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

    #[test]
    /// Tests if the uninstalled DNA is gone from the config file
    /// as well as the instances that use the DNA and their mentions are gone from the interfaces
    /// (to not render the config invalid).
    fn test_uninstall_dna() {
        let mut container = create_test_container("test_uninstall_dna");
        assert_eq!(container.uninstall_dna(&String::from("test-dna")), Ok(()),);

        let mut config_contents = String::new();
        let mut file = File::open(&container.config_path).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        assert_eq!(
            config_contents,
            r#"bridges = []
dnas = []
instances = []
persistence_dir = "./tmp-test/"

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

[[interfaces]]
admin = true
id = "websocket interface"
instances = []

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

    #[test]
    fn test_start_stop_instance() {
        let mut container = create_test_container("test_start_stop_instance");
        assert_eq!(
            container.start_instance(&String::from("test-instance-1")),
            Ok(()),
        );
        assert_eq!(
            container.start_instance(&String::from("test-instance-1")),
            Err(HolochainInstanceError::InstanceAlreadyActive),
        );
        assert_eq!(
            container.start_instance(&String::from("non-existant-id")),
            Err(HolochainInstanceError::NoSuchInstance),
        );
        assert_eq!(
            container.stop_instance(&String::from("test-instance-1")),
            Ok(())
        );
        assert_eq!(
            container.stop_instance(&String::from("test-instance-1")),
            Err(HolochainInstanceError::InstanceNotActiveYet),
        );
    }
}
