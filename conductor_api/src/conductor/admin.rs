use crate::{
    conductor::{base::notify, Conductor},
    config::{
        AgentConfiguration, Bridge, DnaConfiguration, InstanceConfiguration,
        InstanceReferenceConfiguration, InterfaceConfiguration, StorageConfiguration,
    },
    error::HolochainInstanceError,
};
use holochain_core_types::{
    cas::content::AddressableContent, error::HolochainError, hash::HashString,
};
use json_patch;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub trait ConductorAdmin {
    fn install_dna_from_file(
        &mut self,
        path: PathBuf,
        id: String,
        copy: bool,
        expected_hash: Option<HashString>,
        properties: Option<&serde_json::Value>,
    ) -> Result<(), HolochainError>;
    fn uninstall_dna(&mut self, id: &String) -> Result<(), HolochainError>;
    fn add_instance(
        &mut self,
        id: &String,
        dna_id: &String,
        agent_id: &String,
    ) -> Result<(), HolochainError>;
    fn remove_instance(&mut self, id: &String) -> Result<(), HolochainError>;
    fn start_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
    fn stop_instance(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
    fn add_interface(&mut self, new_instance: InterfaceConfiguration)
        -> Result<(), HolochainError>;
    fn remove_interface(&mut self, id: &String) -> Result<(), HolochainError>;
    fn add_instance_to_interface(
        &mut self,
        interface_id: &String,
        instance_id: &String,
    ) -> Result<(), HolochainError>;
    fn remove_instance_from_interface(
        &mut self,
        interface_id: &String,
        instance_id: &String,
    ) -> Result<(), HolochainError>;
    fn add_agent(&mut self, new_agent: AgentConfiguration) -> Result<(), HolochainError>;
    fn remove_agent(&mut self, id: &String) -> Result<(), HolochainError>;
    fn add_bridge(&mut self, new_bridge: Bridge) -> Result<(), HolochainError>;
    fn remove_bridge(
        &mut self,
        caller_id: &String,
        callee_id: &String,
    ) -> Result<(), HolochainError>;
}

impl ConductorAdmin for Conductor {
    /// Installs a DNA package from the file system to the conductor
    /// If copy=true it will also copy the DNA package to the conductors default
    /// location for managing data.
    ///
    /// This function may also take an optional `properties` parameter. This can be any valid JSON
    /// and will be injected in the dna package prior to installation. Existing properties will also be kept and
    /// overriden by the passed properties in the case of collisions. This will change the dna hash!
    /// (Note injecting properties requires that copy=true)
    fn install_dna_from_file(
        &mut self,
        path: PathBuf,
        id: String,
        copy: bool,
        expected_hash: Option<HashString>,
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

        if let Some(hash) = expected_hash {
            if dna.address() != hash {
                return Err(HolochainError::DnaHashMismatch(dna.address(), hash));
            }
        }

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
            hash: Some(dna.address().to_string()),
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

    fn add_instance(
        &mut self,
        id: &String,
        dna_id: &String,
        agent_id: &String,
    ) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        let storage_path = self.instance_storage_dir_path().join(id.clone());
        fs::create_dir_all(&storage_path)?;
        let new_instance_config = InstanceConfiguration {
            id: id.to_string(),
            dna: dna_id.to_string(),
            agent: agent_id.to_string(),
            storage: StorageConfiguration::File {
                path: storage_path
                    .to_str()
                    .ok_or(HolochainError::ConfigError(
                        format!("invalid path {:?}", storage_path).into(),
                    ))?
                    .into(),
            },
        };
        new_config.instances.push(new_instance_config);
        new_config.check_consistency()?;
        let instance = self.instantiate_from_config(id, &new_config, None)?;
        self.instances
            .insert(id.clone(), Arc::new(RwLock::new(instance)));
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
        if new_config.interfaces.iter().any(|i| i.id == interface.id) {
            return Err(HolochainError::ErrorGeneric(format!(
                "Interface with ID '{}' already exists",
                interface.id
            )));
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

        if !new_config
            .interfaces
            .iter()
            .any(|interface| interface.id == *id)
        {
            return Err(HolochainError::ErrorGeneric(format!(
                "No such interface: '{}'",
                id
            )));
        }

        new_config.interfaces = new_config
            .interfaces
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

    fn add_instance_to_interface(
        &mut self,
        interface_id: &String,
        instance_id: &String,
    ) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();

        if new_config
            .interface_by_id(interface_id)
            .ok_or(HolochainError::ErrorGeneric(format!(
                "Interface with ID {} not found",
                interface_id
            )))?
            .instances
            .iter()
            .any(|i| i.id == *instance_id)
        {
            return Err(HolochainError::ErrorGeneric(format!(
                "Instance '{}' already in interface '{}'",
                instance_id, interface_id
            )));
        }

        new_config.interfaces = new_config
            .interfaces
            .into_iter()
            .map(|mut interface| {
                if interface.id == *interface_id {
                    interface.instances.push(InstanceReferenceConfiguration {
                        id: instance_id.clone(),
                    });
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

    fn remove_instance_from_interface(
        &mut self,
        interface_id: &String,
        instance_id: &String,
    ) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();

        if !new_config
            .interface_by_id(interface_id)
            .ok_or(HolochainError::ErrorGeneric(format!(
                "Interface with ID {} not found",
                interface_id
            )))?
            .instances
            .iter()
            .any(|i| i.id == *instance_id)
        {
            return Err(HolochainError::ErrorGeneric(format!(
                "No Instance '{}' in interface '{}'",
                instance_id, interface_id
            )));
        }

        new_config.interfaces = new_config
            .interfaces
            .into_iter()
            .map(|mut interface| {
                if interface.id == *interface_id {
                    interface.instances = interface
                        .instances
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

    fn add_agent(&mut self, new_agent: AgentConfiguration) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        if new_config.agents.iter().any(|i| i.id == new_agent.id) {
            return Err(HolochainError::ErrorGeneric(format!(
                "Agent with ID '{}' already exists",
                new_agent.id
            )));
        }
        new_config.agents.push(new_agent.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        notify(format!("Added agent \"{}\"", new_agent.id));

        Ok(())
    }

    fn remove_agent(&mut self, id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        if !new_config.agents.iter().any(|i| i.id == *id) {
            return Err(HolochainError::ErrorGeneric(format!(
                "Agent with ID '{}' does not exist",
                id
            )));
        }

        new_config.agents = new_config
            .agents
            .into_iter()
            .filter(|agent| agent.id != *id)
            .collect();

        let instance_ids: Vec<String> = new_config
            .instances
            .iter()
            .filter(|instance| instance.agent == *id)
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

        notify(format!("Removed agent \"{}\".", id));

        Ok(())
    }

    fn add_bridge(&mut self, new_bridge: Bridge) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        if new_config
            .bridges
            .iter()
            .any(|b| b.caller_id == new_bridge.caller_id && b.callee_id == new_bridge.callee_id)
        {
            return Err(HolochainError::ErrorGeneric(format!(
                "Bridge from instance '{}' to instance '{}' already exists",
                new_bridge.caller_id, new_bridge.callee_id,
            )));
        }
        new_config.bridges.push(new_bridge.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        notify(format!(
            "Added bridge from '{}' to '{}' as '{}'",
            new_bridge.caller_id, new_bridge.callee_id, new_bridge.handle
        ));

        Ok(())
    }

    fn remove_bridge(
        &mut self,
        caller_id: &String,
        callee_id: &String,
    ) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        if !new_config
            .bridges
            .iter()
            .any(|b| b.caller_id == *caller_id && b.callee_id == *callee_id)
        {
            return Err(HolochainError::ErrorGeneric(format!(
                "Bridge from instance '{}' to instance '{}' does not exist",
                caller_id, callee_id,
            )));
        }

        new_config.bridges = new_config
            .bridges
            .into_iter()
            .filter(|bridge| bridge.caller_id != *caller_id || bridge.callee_id != *callee_id)
            .collect();

        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        notify(format!(
            "Bridge from '{}' to '{}' removed",
            caller_id, callee_id
        ));

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        conductor::base::{
            tests::{example_dna_string, test_key_loader, test_keybundle},
            DnaLoader,
        },
        config::{load_configuration, Configuration, InterfaceConfiguration, InterfaceDriver},
    };
    use holochain_common::paths::DNA_EXTENSION;
    use holochain_core_types::{agent::AgentId, dna::Dna, json::JsonString};
    use std::{
        convert::TryFrom,
        env::current_dir,
        fs::{remove_dir_all, File},
        io::Read,
    };

    pub fn test_dna_loader() -> DnaLoader {
        let loader = Box::new(|_: &PathBuf| {
            Ok(Dna::try_from(JsonString::from(example_dna_string())).unwrap())
        })
            as Box<FnMut(&PathBuf) -> Result<Dna, HolochainError> + Send + Sync>;
        Arc::new(loader)
    }

    pub fn empty_bridges() -> String {
        "bridges = []".to_string()
    }

    pub fn empty_ui_bundles() -> String {
        "ui_bundles = []".to_string()
    }

    pub fn empty_ui_interfaces() -> String {
        "ui_interfaces = []".to_string()
    }

    pub fn persistence_dir(test_name: &str) -> String {
        let persist_dir = current_dir()
            .expect("Could not get current dir")
            .join("tmp-test")
            .join(test_name);
        format!("persistence_dir = \'{}\'", persist_dir.to_str().unwrap()).to_string()
    }

    pub fn header_block(test_name: &str) -> String {
        let mut toml = empty_bridges();
        toml = add_line(toml, persistence_dir(test_name));
        toml = add_line(toml, empty_ui_bundles());
        toml = add_line(toml, empty_ui_interfaces());
        toml
    }

    pub fn agent1() -> String {
        format!(
            r#"[[agents]]
id = 'test-agent-1'
key_file = 'holo_tester1.key'
name = 'Holo Tester 1'
public_address = '{}'"#,
            test_keybundle(1).get_id()
        )
    }

    pub fn agent2() -> String {
        format!(
            r#"[[agents]]
id = 'test-agent-2'
key_file = 'holo_tester2.key'
name = 'Holo Tester 2'
public_address = '{}'"#,
            test_keybundle(2).get_id()
        )
    }

    pub fn dna() -> String {
        r#"[[dnas]]
file = 'app_spec.dna.json'
hash = 'Qm328wyq38924y'
id = 'test-dna'"#
            .to_string()
    }

    pub fn instance1() -> String {
        r#"[[instances]]
agent = 'test-agent-1'
dna = 'test-dna'
id = 'test-instance-1'

[instances.storage]
type = 'memory'"#
            .to_string()
    }

    pub fn instance2() -> String {
        r#"[[instances]]
agent = 'test-agent-2'
dna = 'test-dna'
id = 'test-instance-2'

[instances.storage]
type = 'memory'"#
            .to_string()
    }

    pub fn interface(port: u32) -> String {
        format!(
            r#"[[interfaces]]
admin = true
id = 'websocket interface'

[[interfaces.instances]]
id = 'test-instance-1'

[[interfaces.instances]]
id = 'test-instance-2'

[interfaces.driver]
port = {}
type = 'websocket'"#,
            port
        )
    }

    pub fn logger() -> String {
        r#"[logger]
type = ''
[[logger.rules.rules]]
color = 'red'
exclude = false
pattern = '^err/'

[[logger.rules.rules]]
color = 'white'
exclude = false
pattern = '^debug/dna'

[[logger.rules.rules]]
exclude = false
pattern = '.*'"#
            .to_string()
    }

    pub fn add_block(base: String, new_block: String) -> String {
        format!("{}\n\n{}", base, new_block)
    }

    pub fn add_line(base: String, new_line: String) -> String {
        format!("{}\n{}", base, new_line)
    }

    pub fn test_toml(test_name: &str, port: u32) -> String {
        let mut toml = header_block(test_name);

        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(port));
        toml = add_block(toml, logger());
        toml
    }

    pub fn create_test_conductor(test_name: &str, port: u32) -> Conductor {
        let config = load_configuration::<Configuration>(&test_toml(test_name, port)).unwrap();
        let mut conductor = Conductor::from_config(config.clone());
        conductor.dna_loader = test_dna_loader();
        conductor.key_loader = test_key_loader();
        conductor.load_config().unwrap();
        conductor
    }

    #[test]
    fn test_install_dna_from_file() {
        let test_name = "test_install_dna_from_file";
        let mut conductor = create_test_conductor(test_name, 3000);

        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.dna.json");

        assert_eq!(
            conductor.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna"),
                false,
                None,
                None
            ),
            Ok(()),
        );

        let new_dna =
            Arc::get_mut(&mut test_dna_loader()).unwrap()(&PathBuf::from("new-dna.dna.json"))
                .unwrap();

        assert_eq!(conductor.config().dnas.len(), 2,);

        assert_eq!(
            conductor.config().dnas,
            vec![
                DnaConfiguration {
                    id: String::from("test-dna"),
                    file: String::from("app_spec.dna.json"),
                    hash: Some(String::from("Qm328wyq38924y")),
                },
                DnaConfiguration {
                    id: String::from("new-dna"),
                    file: String::from("new-dna.dna.json"),
                    hash: Some(String::from(new_dna.address())),
                },
            ]
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = header_block(test_name);
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(
            toml,
            String::from(
                r#"[[dnas]]
file = 'new-dna.dna.json'
hash = 'QmQVLgFxUpd1ExVkBzvwASshpG6fmaJGxDEgf1cFf7S73a'
id = 'new-dna'"#,
            ),
        );
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(3000));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);
    }

    #[test]
    fn test_install_dna_from_file_and_copy() {
        let test_name = "test_install_dna_from_file_and_copy";
        let mut conductor = create_test_conductor(test_name, 3000);

        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.dna.json");

        assert_eq!(
            conductor.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna"),
                true,
                None,
                None
            ),
            Ok(()),
        );

        let new_dna =
            Arc::get_mut(&mut test_dna_loader()).unwrap()(&PathBuf::from("new-dna.dna.json"))
                .unwrap();

        assert_eq!(conductor.config().dnas.len(), 2,);

        let mut output_dna_file = current_dir()
            .expect("Could not get current dir")
            .join("tmp-test")
            .join(test_name)
            .join("dna");

        output_dna_file.push(new_dna.address().to_string());
        output_dna_file.set_extension(DNA_EXTENSION);

        assert_eq!(
            conductor.config().dnas,
            vec![
                DnaConfiguration {
                    id: String::from("test-dna"),
                    file: String::from("app_spec.dna.json"),
                    hash: Some(String::from("Qm328wyq38924y")),
                },
                DnaConfiguration {
                    id: String::from("new-dna"),
                    file: output_dna_file.to_str().unwrap().to_string(),
                    hash: Some(String::from(new_dna.address())),
                },
            ]
        );
        assert!(output_dna_file.is_file())
    }

    #[test]
    fn test_install_dna_with_expected_hash() {
        let test_name = "test_install_dna_with_expected_hash";
        let mut conductor = create_test_conductor(test_name, 3000);
        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.dna.json");
        let dna = Arc::get_mut(&mut conductor.dna_loader).unwrap()(&new_dna_path).unwrap();

        assert_eq!(
            conductor.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna"),
                false,
                Some(dna.address()),
                None
            ),
            Ok(()),
        );

        assert_eq!(
            conductor.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna"),
                false,
                Some("wrong-address".into()),
                None
            ),
            Err(HolochainError::DnaHashMismatch(
                dna.address(),
                "wrong-address".into()
            )),
        );
    }

    #[test]
    fn test_install_dna_from_file_with_properties() {
        let test_name = "test_install_dna_from_file_with_properties";
        let mut conductor = create_test_conductor(test_name, 3000);

        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.dna.json");
        let new_props = json!({"propertyKey": "value"});

        assert_eq!(
            conductor.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna-with-props"),
                false,
                None,
                Some(&new_props)
            ),
            Err(HolochainError::ConfigError(
                "Cannot install DNA with properties unless copy flag is true".into()
            )),
        );

        assert_eq!(
            conductor.install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna-with-props"),
                true,
                None,
                Some(&new_props)
            ),
            Ok(()),
        );

        let mut new_dna =
            Arc::get_mut(&mut test_dna_loader()).unwrap()(&PathBuf::from("new-dna.dna.json"))
                .unwrap();
        let original_hash = new_dna.address();
        new_dna.properties = new_props;
        let new_hash = new_dna.address();
        assert_ne!(original_hash, new_hash);
        assert_eq!(conductor.config().dnas.len(), 2,);

        let mut output_dna_file = current_dir()
            .expect("Could not get current dir")
            .join("tmp-test")
            .join(test_name)
            .join("dna");

        output_dna_file.push(new_hash.to_string());
        output_dna_file.set_extension(DNA_EXTENSION);

        assert_eq!(
            conductor.config().dnas,
            vec![
                DnaConfiguration {
                    id: String::from("test-dna"),
                    file: String::from("app_spec.dna.json"),
                    hash: Some(String::from("Qm328wyq38924y")),
                },
                DnaConfiguration {
                    id: String::from("new-dna-with-props"),
                    file: output_dna_file.to_str().unwrap().to_string(),
                    hash: Some(String::from(new_dna.address())),
                },
            ]
        );
        assert!(output_dna_file.is_file())
    }

    #[test]
    fn test_add_instance() {
        let test_name = "test_add_instance";
        let mut conductor = create_test_conductor(test_name, 3001);

        let mut new_dna_path = PathBuf::new();
        new_dna_path.push("new-dna.dna.json");
        conductor
            .install_dna_from_file(
                new_dna_path.clone(),
                String::from("new-dna"),
                false,
                None,
                None,
            )
            .expect("Could not install DNA");

        let add_result = conductor.add_instance(
            &String::from("new-instance"),
            &String::from("new-dna"),
            &String::from("test-agent-1"),
        );

        assert_eq!(add_result, Ok(()));

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = header_block(test_name);
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(
            toml,
            String::from(
                r#"[[dnas]]
file = 'new-dna.dna.json'
hash = 'QmQVLgFxUpd1ExVkBzvwASshpG6fmaJGxDEgf1cFf7S73a'
id = 'new-dna'"#,
            ),
        );
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(
            toml,
            String::from(
                r#"[[instances]]
agent = 'test-agent-1'
dna = 'new-dna'
id = 'new-instance'"#,
            ),
        );

        let storage_path = current_dir()
            .expect("Could not get current dir")
            .join("tmp-test")
            .join(test_name)
            .join("storage")
            .join("new-instance");

        let storage_path_string = storage_path.to_str().unwrap().to_owned();
        toml = add_block(
            toml,
            format!(
                "[instances.storage]\npath = '{}'\ntype = 'file'",
                storage_path_string
            ),
        );
        toml = add_block(toml, interface(3001));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);
    }

    #[test]
    /// Tests if the removed instance is gone from the config file
    /// as well as the mentions of the removed instance are gone from the interfaces
    /// (to not render the config invalid).
    fn test_remove_instance() {
        let test_name = "test_remove_instance";
        let mut conductor = create_test_conductor(test_name, 3002);

        assert_eq!(
            conductor.remove_instance(&String::from("test-instance-1")),
            Ok(()),
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = header_block(test_name);

        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        //toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(
            toml,
            String::from(
                r#"[[interfaces]]
admin = true
id = 'websocket interface'

[[interfaces.instances]]
id = 'test-instance-2'

[interfaces.driver]
port = 3002
type = 'websocket'"#,
            ),
        );
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);
    }

    #[test]
    /// Tests if the uninstalled DNA is gone from the config file
    /// as well as the instances that use the DNA and their mentions are gone from the interfaces
    /// (to not render the config invalid).
    fn test_uninstall_dna() {
        let test_name = "test_uninstall_dna";
        let mut conductor = create_test_conductor(test_name, 3003);

        assert_eq!(conductor.uninstall_dna(&String::from("test-dna")), Ok(()),);

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = empty_bridges();
        toml = add_line(toml, "dnas = []".to_string());
        toml = add_line(toml, "instances = []".to_string());
        toml = add_line(toml, persistence_dir(test_name));
        toml = add_line(toml, empty_ui_bundles());
        toml = add_line(toml, empty_ui_interfaces());

        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        //toml = add_block(toml, dna());
        //toml = add_block(toml, instance1());
        //toml = add_block(toml, instance2());
        toml = add_block(
            toml,
            String::from(
                r#"[[interfaces]]
admin = true
id = 'websocket interface'
instances = []

[interfaces.driver]
port = 3003
type = 'websocket'"#,
            ),
        );
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);
    }

    #[test]
    fn test_start_stop_instance() {
        let mut conductor = create_test_conductor("test_start_stop_instance", 3004);
        assert_eq!(
            conductor.start_instance(&String::from("test-instance-1")),
            Ok(()),
        );
        assert_eq!(
            conductor.start_instance(&String::from("test-instance-1")),
            Err(HolochainInstanceError::InstanceAlreadyActive),
        );
        assert_eq!(
            conductor.start_instance(&String::from("non-existant-id")),
            Err(HolochainInstanceError::NoSuchInstance),
        );
        assert_eq!(
            conductor.stop_instance(&String::from("test-instance-1")),
            Ok(())
        );
        assert_eq!(
            conductor.stop_instance(&String::from("test-instance-1")),
            Err(HolochainInstanceError::InstanceNotActiveYet),
        );
    }

    #[test]
    fn test_add_interface() {
        let test_name = "test_add_interface";
        let mut conductor = create_test_conductor(test_name, 3005);

        let interface_config = InterfaceConfiguration {
            id: String::from("new-interface"),
            driver: InterfaceDriver::Http { port: 8080 },
            admin: false,
            instances: Vec::new(),
        };

        assert_eq!(conductor.add_interface(interface_config), Ok(()),);

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = header_block(test_name);
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(3005));
        toml = add_block(
            toml,
            String::from(
                r#"[[interfaces]]
admin = false
id = 'new-interface'
instances = []

[interfaces.driver]
port = 8080
type = 'http'"#,
            ),
        );
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);
    }

    #[test]
    fn test_remove_interface() {
        let test_name = "test_remove_interface";
        let mut conductor = create_test_conductor(test_name, 3006);

        conductor.start_all_interfaces();
        assert!(conductor
            .interface_threads
            .get("websocket interface")
            .is_some());

        assert_eq!(
            conductor.remove_interface(&String::from("websocket interface")),
            Ok(())
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = empty_bridges();
        toml = add_line(toml, "interfaces = []".to_string());
        toml = add_line(toml, persistence_dir(test_name));
        toml = add_line(toml, empty_ui_bundles());
        toml = add_line(toml, empty_ui_interfaces());

        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);

        assert!(conductor
            .interface_threads
            .get("websocket interface")
            .is_none());
    }

    #[test]
    fn test_add_instance_to_interface() {
        let test_name = "test_add_instance_to_interface";
        let mut conductor = create_test_conductor(test_name, 3007);

        let storage_path = current_dir()
            .expect("Could not get current dir")
            .join("tmp-test")
            .join(test_name)
            .join("storage")
            .join("new-instance-2");

        // Make sure storage is clean
        remove_dir_all(storage_path.clone()).expect("Could not clear storage directory");

        conductor.start_all_interfaces();
        assert!(conductor
            .interface_threads
            .get("websocket interface")
            .is_some());

        assert_eq!(
            conductor.add_instance(
                &String::from("new-instance-2"),
                &String::from("test-dna"),
                &String::from("test-agent-1")
            ),
            Ok(())
        );

        assert_eq!(
            conductor.add_instance_to_interface(
                &String::from("websocket interface"),
                &String::from("new-instance-2")
            ),
            Ok(())
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = header_block(test_name);
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(
            toml,
            String::from(
                r#"[[instances]]
agent = 'test-agent-1'
dna = 'test-dna'
id = 'new-instance-2'"#,
            ),
        );

        let storage_path_string = storage_path.to_str().unwrap().to_owned();
        toml = add_block(
            toml,
            format!(
                "[instances.storage]\npath = '{}'\ntype = 'file'",
                storage_path_string
            ),
        );
        toml = add_block(
            toml,
            String::from(
                r#"[[interfaces]]
admin = true
id = 'websocket interface'

[[interfaces.instances]]
id = 'test-instance-1'

[[interfaces.instances]]
id = 'test-instance-2'

[[interfaces.instances]]
id = 'new-instance-2'

[interfaces.driver]
port = 3007
type = 'websocket'"#,
            ),
        );
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);
    }

    #[test]
    fn test_remove_instance_from_interface() {
        let test_name = "test_remove_instance_from_interface";
        let mut conductor = create_test_conductor(test_name, 3008);

        conductor.start_all_interfaces();
        assert!(conductor
            .interface_threads
            .get("websocket interface")
            .is_some());

        assert_eq!(
            conductor.remove_instance_from_interface(
                &String::from("websocket interface"),
                &String::from("test-instance-1")
            ),
            Ok(())
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = header_block(test_name);
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(
            toml,
            String::from(
                r#"[[interfaces]]
admin = true
id = 'websocket interface'

[[interfaces.instances]]
id = 'test-instance-2'

[interfaces.driver]
port = 3008
type = 'websocket'"#,
            ),
        );
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);

        assert!(conductor
            .interface_threads
            .get("websocket interface")
            .is_some());
    }

    #[test]
    fn test_add_agent() {
        let test_name = "test_add_agent";
        let mut conductor = create_test_conductor(test_name, 3009);

        let agent_config = AgentConfiguration {
            id: String::from("new-agent"),
            name: String::from("Mr. New"),
            public_address: AgentId::generate_fake("new").address().to_string(),
            key_file: String::from("new-test-path"),
            holo_remote_key: None,
        };

        assert_eq!(conductor.add_agent(agent_config), Ok(()),);

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = header_block(test_name);
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(
            toml,
            String::from(
                r#"[[agents]]
id = 'new-agent'
key_file = 'new-test-path'
name = 'Mr. New'
public_address = 'HcScIkRaAaaaaaaaaaAaaaAAAAaaaaaaaaAaaaaAaaaaaaaaAaaAAAAatzu4aqa'"#,
            ),
        );
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(3009));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);
    }

    #[test]
    fn test_remove_agent() {
        let test_name = "test_remove_agent";
        let mut conductor = create_test_conductor(test_name, 3010);

        assert_eq!(
            conductor.remove_agent(&String::from("test-agent-2")),
            Ok(()),
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = header_block(test_name);
        toml = add_block(toml, agent1());
        //toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        //toml = add_block(toml, instance2());
        //toml = add_block(toml, interface());
        toml = add_block(
            toml,
            String::from(
                r#"[[interfaces]]
admin = true
id = 'websocket interface'

[[interfaces.instances]]
id = 'test-instance-1'

[interfaces.driver]
port = 3010
type = 'websocket'"#,
            ),
        );
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);
    }

    #[test]
    fn test_add_and_remove_bridge() {
        let test_name = "test_add_and_remove_bridge";
        let mut conductor = create_test_conductor(test_name, 3011);

        let bridge = Bridge {
            caller_id: String::from("test-instance-1"),
            callee_id: String::from("test-instance-2"),
            handle: String::from("my favourite instance!"),
        };

        assert_eq!(conductor.add_bridge(bridge), Ok(()),);

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = persistence_dir(test_name);
        toml = add_line(toml, empty_ui_bundles());
        toml = add_line(toml, empty_ui_interfaces());

        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(
            toml,
            String::from(
                r#"[[bridges]]
callee_id = 'test-instance-2'
caller_id = 'test-instance-1'
handle = 'my favourite instance!'"#,
            ),
        );
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(3011));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);

        assert_eq!(
            conductor.remove_bridge(
                &String::from("test-instance-1"),
                &String::from("test-instance-2")
            ),
            Ok(()),
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = header_block(test_name);
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(3011));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,);
    }
}
