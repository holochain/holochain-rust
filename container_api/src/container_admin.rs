use crate::{
    config::{
        DnaConfiguration, InstanceConfiguration,
        InstanceReferenceConfiguration, InterfaceConfiguration,
    },
    container::{notify, Container},
    error::HolochainInstanceError,
};
use holochain_core_types::{cas::content::AddressableContent, error::HolochainError};
use std::{path::PathBuf, sync::Arc};

pub trait ContainerAdmin {
    fn install_dna_from_file(&mut self, path: PathBuf, id: String) -> Result<(), HolochainError>;
    fn uninstall_dna(&mut self, id: &String) -> Result<(), HolochainError>;
    fn add_instance(&mut self, new_instance: InstanceConfiguration) -> Result<(), HolochainError>;
    fn remove_instance(&mut self, id: &String) -> Result<(), HolochainError>;
    fn start_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
    fn stop_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
    fn add_interface(&mut self, new_instance: InterfaceConfiguration) -> Result<(), HolochainError>;
    fn remove_interface(&mut self, id: &String) -> Result<(), HolochainError>;
    fn add_instance_to_interface(&mut self, interface_id: &String, instance_id: &String) -> Result<(), HolochainError>;
    fn remove_instance_from_interface(&mut self, interface_id: &String, instance_id: &String) -> Result<(), HolochainError>;
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

    fn add_interface(&mut self, interface: InterfaceConfiguration) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        if new_config.interfaces
            .iter()
            .find(|i| i.id == interface.id)
            .is_some() {
            return Err(HolochainError::ErrorGeneric(format!("Interface with ID '{}' already exists", interface.id)))
        }
        new_config.interfaces.push(interface.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        self.start_interface_by_id(&interface.id)?;
        Ok(())
    }

    fn remove_interface(&mut self, id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();

        if new_config.interfaces
            .iter()
            .find(|interface| interface.id == *id)
            .is_none() {
            return Err(HolochainError::ErrorGeneric(format!("No such interface: '{}'", id)))
        }

        new_config.interfaces = new_config.interfaces
            .into_iter()
            .filter(|interface| interface.id != *id)
            .collect();

        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        let _ = self.stop_interface_by_id(id);

        notify(format!("Removed interface \"{}\".", id));
        Ok(())
    }

    fn add_instance_to_interface(&mut self, interface_id: &String, instance_id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();

        if new_config.interface_by_id(interface_id)
            .ok_or(HolochainError::ErrorGeneric(format!("Instance with ID {} not found", instance_id)))?
            .instances
            .iter()
            .find(|i| i.id == *instance_id)
            .is_some()
        {
            return Err(HolochainError::ErrorGeneric(
                format!("Instance '{}' already in interface '{}'", instance_id, interface_id))
            )
        }

        new_config.interfaces = new_config.interfaces
            .into_iter()
            .map(|mut interface| {
                if interface.id == *interface_id {
                    interface.instances.push(
                        InstanceReferenceConfiguration{id: instance_id.clone()}
                    );
                }
                interface
            })
            .collect();

        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        let _ = self.stop_interface_by_id(interface_id);
        self.start_interface_by_id(interface_id)?;

        Ok(())
    }

    fn remove_instance_from_interface(&mut self, interface_id: &String, instance_id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();

        if new_config.interface_by_id(interface_id)
            .ok_or(HolochainError::ErrorGeneric(format!("Instance with ID {} not found", instance_id)))?
            .instances
            .iter()
            .find(|i| i.id == *instance_id)
            .is_none()
        {
            return Err(HolochainError::ErrorGeneric(
                format!("No Instance '{}' in interface '{}'", instance_id, interface_id))
            )
        }

        new_config.interfaces = new_config.interfaces
            .into_iter()
            .map(|mut interface| {
                if interface.id == *interface_id {
                    interface.instances = interface.instances
                        .into_iter()
                        .filter(|instance| instance.id != *instance_id)
                        .collect();
                }
                interface
            })
            .collect();

        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        let _ = self.stop_interface_by_id(interface_id);
        self.start_interface_by_id(interface_id)?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        config::{load_configuration, Configuration, InterfaceConfiguration, InterfaceDriver},
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

    pub fn agent1() -> String {
r#"[[agents]]
id = "test-agent-1"
key_file = "holo_tester.key"
name = "Holo Tester 1"
public_address = "HoloTester1-----------------------------------------------------------------------AAACZp4xHB""#
    .to_string()
    }

    pub fn agent2() -> String {
r#"[[agents]]
id = "test-agent-2"
key_file = "holo_tester.key"
name = "Holo Tester 2"
public_address = "HoloTester2-----------------------------------------------------------------------AAAGy4WW9e""#
    .to_string()
    }

    pub fn dna() -> String {
r#"[[dnas]]
file = "app_spec.hcpkg"
hash = "Qm328wyq38924y"
id = "test-dna""#
            .to_string()
    }


    pub fn instance1() -> String {
r#"[[instances]]
agent = "test-agent-1"
dna = "test-dna"
id = "test-instance-1"

[instances.storage]
type = "memory""#
            .to_string()
    }

    pub fn instance2() -> String {
r#"[[instances]]
agent = "test-agent-2"
dna = "test-dna"
id = "test-instance-2"

[instances.storage]
type = "memory""#
            .to_string()
    }

    pub fn interface() -> String {
r#"[[interfaces]]
admin = true
id = "websocket interface"

[[interfaces.instances]]
id = "test-instance-1"

[[interfaces.instances]]
id = "test-instance-2"

[interfaces.driver]
port = 3000
type = "websocket""#
    .to_string()
    }

    pub fn logger() -> String {
r#"[logger]
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
pattern = ".*""#
    .to_string()
    }

    fn add_block(base: String, new_block: String) -> String {
        format!("{}\n\n{}", base, new_block)
    }

    pub fn test_toml() -> String {
        let mut toml = String::from("bridges = []");
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface());
        toml = add_block(toml, logger());
        toml
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

        let mut toml = String::from("bridges = []");
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, String::from(
r#"[[dnas]]
file = "new-dna.hcpkg"
hash = "QmPB7PJUjwj6zap7jB7oyk616sCRSSnNFRNouqhit6kMTr"
id = "new-dna""#
        ));
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface());
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(
            config_contents,
            toml,
        );
    }

    use crate::config::StorageConfiguration;
    #[test]
    fn test_add_instance() {
        let mut container = create_test_container("test_add_instance");
        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.hcpkg");
        container
            .install_dna_from_file(new_dna_path.clone(), String::from("new-dna"))
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

        let mut toml = String::from("bridges = []");
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, String::from(
            r#"[[dnas]]
file = "new-dna.hcpkg"
hash = "QmPB7PJUjwj6zap7jB7oyk616sCRSSnNFRNouqhit6kMTr"
id = "new-dna""#
        ));
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, String::from(
r#"[[instances]]
agent = "test-agent-1"
dna = "new-dna"
id = "new-instance"

[instances.storage]
type = "memory""#
        ));
        toml = add_block(toml, interface());
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(
            config_contents,
            toml,
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

        let mut toml = String::from("bridges = []");
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        //toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, String::from(
r#"[[interfaces]]
admin = true
id = "websocket interface"

[[interfaces.instances]]
id = "test-instance-2"

[interfaces.driver]
port = 3000
type = "websocket""#
        ));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(
            config_contents,
            toml,
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

        let mut toml = String::from("bridges = []\ndnas = []\ninstances = []");
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        //toml = add_block(toml, dna());
        //toml = add_block(toml, instance1());
        //toml = add_block(toml, instance2());
        toml = add_block(toml, String::from(
r#"[[interfaces]]
admin = true
id = "websocket interface"
instances = []

[interfaces.driver]
port = 3000
type = "websocket""#
        ));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(
            config_contents,
            toml,
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

    #[test]
    fn test_add_interface() {
        let mut container = create_test_container("test_add_interface");
        let interface_config = InterfaceConfiguration {
            id: String::from("new-interface"),
            driver: InterfaceDriver::Http { port: 8080 },
            admin: false,
            instances: Vec::new(),
        };

        assert_eq!(container.add_interface(interface_config), Ok(()), );

        let mut config_contents = String::new();
        let mut file = File::open(&container.config_path).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = String::from("bridges = []");
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface());
        toml = add_block(toml, String::from(
            r#"[[interfaces]]
admin = false
id = "new-interface"
instances = []

[interfaces.driver]
port = 8080
type = "http""#
        ));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(
            config_contents,
            toml,
        );
    }

    #[test]
    fn test_remove_interface() {
        let mut container = create_test_container("test_remove_interface");
        container.start_all_interfaces();
        assert!(container.interface_threads.get("websocket interface").is_some());

        assert_eq!(container.remove_interface(&String::from("websocket interface")), Ok(()));

        let mut config_contents = String::new();
        let mut file = File::open(&container.config_path).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = String::from("bridges = []\ninterfaces = []");
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(
            config_contents,
            toml,
        );

        assert!(container.interface_threads.get("websocket interface").is_none());
    }

    #[test]
    fn test_add_instance_to_interface() {
        let mut container = create_test_container("test_add_instance_to_interface");
        container.start_all_interfaces();
        assert!(container.interface_threads.get("websocket interface").is_some());

        let instance_config = InstanceConfiguration {
            id: String::from("new-instance"),
            dna: String::from("test-dna"),
            agent: String::from("test-agent-1"),
            storage: StorageConfiguration::Memory,
        };

        assert_eq!(container.add_instance(instance_config.clone()), Ok(()));
        assert_eq!(
            container.add_instance_to_interface(
            &String::from("websocket interface"),
            &String::from("new-instance")
            ),
            Ok(()));

        let mut config_contents = String::new();
        let mut file = File::open(&container.config_path).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = String::from("bridges = []");
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, String::from(
r#"[[instances]]
agent = "test-agent-1"
dna = "test-dna"
id = "new-instance"

[instances.storage]
type = "memory""#
        ));
        toml = add_block(toml, String::from(
r#"[[interfaces]]
admin = true
id = "websocket interface"

[[interfaces.instances]]
id = "test-instance-1"

[[interfaces.instances]]
id = "test-instance-2"

[[interfaces.instances]]
id = "new-instance"

[interfaces.driver]
port = 3000
type = "websocket""#
        ));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(
            config_contents,
            toml,
        );
    }

    #[test]
    fn test_remove_instance_from_interface() {
        let mut container = create_test_container("test_remove_instance_from_interface");
        container.start_all_interfaces();
        assert!(container.interface_threads.get("websocket interface").is_some());

        assert_eq!(
            container.remove_instance_from_interface(
                &String::from("websocket interface"),
                &String::from("test-instance-1")
            ),
            Ok(()));

        let mut config_contents = String::new();
        let mut file = File::open(&container.config_path).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = String::from("bridges = []");
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, String::from(
r#"[[interfaces]]
admin = true
id = "websocket interface"

[[interfaces.instances]]
id = "test-instance-2"

[interfaces.driver]
port = 3000
type = "websocket""#
        ));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(
            config_contents,
            toml,
        );

        assert!(container.interface_threads.get("websocket interface").is_some());
    }
}
