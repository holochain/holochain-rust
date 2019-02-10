use crate::{
    conductor::{base::notify, Conductor},
    config::{UiBundleConfiguration, UiInterfaceConfiguration},
    static_file_server::StaticServer,
};
use error::HolochainInstanceError;
use holochain_core_types::error::HolochainError;
use std::{path::PathBuf, sync::Arc};

pub trait ConductorUiAdmin {
    fn install_ui_bundle_from_file(
        &mut self,
        path: PathBuf,
        id: &String,
        copy: bool,
    ) -> Result<(), HolochainError>;
    fn uninstall_ui_bundle(&mut self, id: &String) -> Result<(), HolochainError>;

    fn add_ui_interface(
        &mut self,
        new_instance: UiInterfaceConfiguration,
    ) -> Result<(), HolochainError>;
    fn remove_ui_interface(&mut self, id: &String) -> Result<(), HolochainError>;

    fn start_ui_interface(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
    fn stop_ui_interface(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
}

impl ConductorUiAdmin for Conductor {
    fn install_ui_bundle_from_file(
        &mut self,
        path: PathBuf,
        id: &String,
        copy: bool,
    ) -> Result<(), HolochainError> {
        let path = match copy {
            true => {
                let dest = self.config.persistence_dir.join("static").join(id);

                Arc::get_mut(&mut self.ui_dir_copier).unwrap()(&path, &dest).map_err(|e| {
                    HolochainError::ErrorGeneric(format!(
                        "Error copying DNA from {} to {}: {}",
                        path.display(),
                        dest.display(),
                        e
                    ))
                })?;
                dest
            }
            false => path,
        };

        let path_string = path
            .to_str()
            .ok_or(HolochainError::ConfigError("invalid path".into()))?;

        let new_bundle = UiBundleConfiguration {
            id: id.to_string(),
            root_dir: path_string.into(),
            hash: "<not-used>".to_string(),
        };

        let mut new_config = self.config.clone();
        new_config.ui_bundles.push(new_bundle.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        notify(format!(
            "Installed UI bundle from {} as \"{}\"",
            path_string, id
        ));
        Ok(())
    }

    /// Removes the UI bundle in the config.
    /// Also stops then removes its UI interface if any exist
    fn uninstall_ui_bundle(&mut self, id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        new_config.ui_bundles = new_config
            .ui_bundles
            .into_iter()
            .filter(|bundle| bundle.id != *id)
            .collect();

        if new_config.ui_bundles.len() == self.config.ui_bundles.len() {
            return Err(HolochainError::ConfigError(format!(
                "No UI bundles match the given ID \"{}\"",
                id
            )));
        }

        let to_remove = new_config
            .ui_interfaces
            .clone()
            .into_iter()
            .filter(|ui_interface| ui_interface.bundle == id.to_string());

        for bundle_interface in to_remove {
            self.remove_ui_interface(&bundle_interface.id)?;
        }

        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        Ok(())
    }

    fn add_ui_interface(
        &mut self,
        new_interface: UiInterfaceConfiguration,
    ) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        new_config.ui_interfaces.push(new_interface.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        self.static_servers.insert(
            new_interface.id.clone(),
            StaticServer::from_configs(
                new_interface.clone(),
                self.config.ui_bundle_by_id(&new_interface.bundle).unwrap(),
                None,
            ),
        );
        Ok(())
    }

    fn remove_ui_interface(&mut self, id: &String) -> Result<(), HolochainError> {
        let to_stop = self
            .config
            .clone()
            .ui_interfaces
            .into_iter()
            .filter(|ui_interface| ui_interface.id == *id);

        for ui_interface in to_stop {
            let _ = self.stop_ui_interface(&ui_interface.id);
        }

        let mut new_config = self.config.clone();
        new_config.ui_interfaces = new_config
            .ui_interfaces
            .into_iter()
            .filter(|ui_interface| ui_interface.id != *id)
            .collect();
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;

        self.static_servers
            .remove(id)
            .ok_or(HolochainError::ErrorGeneric(
                "Could not remove server".into(),
            ))?;

        Ok(())
    }

    fn start_ui_interface(&mut self, id: &String) -> Result<(), HolochainInstanceError> {
        let server = self.static_servers.get_mut(id)?;
        notify(format!("Starting UI interface \"{}\"...", id));
        server.start()
    }

    fn stop_ui_interface(&mut self, id: &String) -> Result<(), HolochainInstanceError> {
        let server = self.static_servers.get_mut(id)?;
        notify(format!("Stopping UI interface \"{}\"...", id));
        server.stop()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use conductor::{admin::tests::*, base::UiDirCopier};
    use std::{fs::File, io::Read};

    pub fn test_ui_copier() -> UiDirCopier {
        let copier = Box::new(|_source: &PathBuf, _dest: &PathBuf| Ok(()))
            as Box<FnMut(&PathBuf, &PathBuf) -> Result<(), HolochainError> + Send + Sync>;
        Arc::new(copier)
    }

    #[test]
    fn test_install_ui_bundle_from_file() {
        let test_name = "test_install_ui_bundle_from_file";
        let mut conductor = create_test_conductor(test_name, 3000);
        let bundle_path = PathBuf::from(".");
        assert_eq!(
            conductor.install_ui_bundle_from_file(
                bundle_path,
                &"test-bundle-id".to_string(),
                false
            ),
            Ok(())
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = empty_bridges();
        toml = add_line(toml, persistence_dir(test_name));
        toml = add_line(toml, empty_ui_interfaces());
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(3000));
        toml = add_block(
            toml,
            String::from(
                r#"[[ui_bundles]]
hash = '<not-used>'
id = 'test-bundle-id'
root_dir = '.'"#,
            ),
        );
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,)
    }

    #[test]
    fn test_install_ui_bundle_from_file_and_copy() {
        let test_name = "test_install_ui_bundle_from_file_and_copy";
        let mut conductor = create_test_conductor(test_name, 3100);

        conductor.ui_dir_copier = test_ui_copier();

        let bundle_path = PathBuf::from(".");
        assert_eq!(
            conductor.install_ui_bundle_from_file(bundle_path, &"test-bundle-id".to_string(), true),
            Ok(())
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let dest = conductor
            .config
            .persistence_dir
            .join("static")
            .join("test-bundle-id");

        let mut toml = empty_bridges();
        toml = add_line(toml, persistence_dir(test_name));
        toml = add_line(toml, empty_ui_interfaces());
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(3100));
        toml = add_block(
            toml,
            String::from(
                r#"[[ui_bundles]]
hash = '<not-used>'
id = 'test-bundle-id'"#,
            ),
        );
        toml = add_line(toml, format!("root_dir = '{}'", dest.display()));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,)
    }

    #[test]
    fn test_uninstall_ui_bundle() {
        let test_name = "test_uninstall_ui_bundle";
        let mut conductor = create_test_conductor(test_name, 3001);
        assert_eq!(
            conductor.uninstall_ui_bundle(&"test-bundle-id".to_string()),
            Err(HolochainError::ConfigError(
                "No UI bundles match the given ID \"test-bundle-id\"".into()
            ))
        );
        let bundle_path = PathBuf::from(".");
        assert_eq!(
            conductor.install_ui_bundle_from_file(
                bundle_path,
                &"test-bundle-id".to_string(),
                false
            ),
            Ok(())
        );
        assert_eq!(
            conductor.uninstall_ui_bundle(&"test-bundle-id".to_string()),
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
        toml = add_block(toml, interface(3001));
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml,)
    }

    #[test]
    fn test_add_ui_interface() {
        let test_name = "test_add_ui_interface";
        let mut conductor = create_test_conductor(test_name, 3002);
        assert_eq!(
            conductor.add_ui_interface(UiInterfaceConfiguration {
                id: "test-ui-interface-id".into(),
                port: 4000,
                bundle: "test-bundle-id".into(),
                dna_interface: None,
            }),
            Err(HolochainError::ErrorGeneric(
                "UI bundle configuration test-bundle-id not found, mentioned in UI interface test-ui-interface-id".into()
            ))
        );

        let bundle_path = PathBuf::from(".");
        assert_eq!(
            conductor.install_ui_bundle_from_file(
                bundle_path,
                &"test-bundle-id".to_string(),
                false
            ),
            Ok(())
        );

        assert_eq!(
            conductor.add_ui_interface(UiInterfaceConfiguration {
                id: "test-ui-interface-id".into(),
                port: 4000,
                bundle: "test-bundle-id".into(),
                dna_interface: None,
            }),
            Ok(())
        );
        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = empty_bridges();
        toml = add_line(toml, persistence_dir(test_name));
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(3002));
        toml = add_block(
            toml,
            String::from(
                r#"[[ui_bundles]]
hash = '<not-used>'
id = 'test-bundle-id'
root_dir = '.'

[[ui_interfaces]]
bundle = 'test-bundle-id'
id = 'test-ui-interface-id'
port = 4000"#,
            ),
        );
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml);
    }

    #[test]
    fn test_remove_ui_interface() {
        let test_name = "test_remove_ui_interface";
        let mut conductor = create_test_conductor(test_name, 3003);

        assert_eq!(
            conductor.remove_ui_interface(&"test-ui-interface-id".to_string()),
            Err(HolochainError::ErrorGeneric(
                "Could not remove server".into()
            ))
        );

        let bundle_path = PathBuf::from(".");
        assert_eq!(
            conductor.install_ui_bundle_from_file(
                bundle_path,
                &"test-bundle-id".to_string(),
                false
            ),
            Ok(())
        );

        assert_eq!(
            conductor.add_ui_interface(UiInterfaceConfiguration {
                id: "test-ui-interface-id".into(),
                port: 4000,
                bundle: "test-bundle-id".into(),
                dna_interface: None,
            }),
            Ok(())
        );

        assert_eq!(
            conductor.remove_ui_interface(&"test-ui-interface-id".to_string()),
            Ok(())
        );

        let mut config_contents = String::new();
        let mut file =
            File::open(&conductor.config_path()).expect("Could not open temp config file");
        file.read_to_string(&mut config_contents)
            .expect("Could not read temp config file");

        let mut toml = empty_bridges();
        toml = add_line(toml, persistence_dir(test_name));
        toml = add_line(toml, empty_ui_interfaces());
        toml = add_block(toml, agent1());
        toml = add_block(toml, agent2());
        toml = add_block(toml, dna());
        toml = add_block(toml, instance1());
        toml = add_block(toml, instance2());
        toml = add_block(toml, interface(3003));
        toml = add_block(
            toml,
            String::from(
                r#"[[ui_bundles]]
hash = '<not-used>'
id = 'test-bundle-id'
root_dir = '.'"#,
            ),
        );
        toml = add_block(toml, logger());
        toml = format!("{}\n", toml);

        assert_eq!(config_contents, toml);
    }

    #[test]
    fn test_start_ui_interface() {
        let test_name = "test_start_ui_interface";
        let mut conductor = create_test_conductor(test_name, 3004);

        let bundle_path = PathBuf::from(".");
        assert_eq!(
            conductor.install_ui_bundle_from_file(
                bundle_path,
                &"test-bundle-id".to_string(),
                false
            ),
            Ok(())
        );

        assert_eq!(
            conductor.add_ui_interface(UiInterfaceConfiguration {
                id: "test-ui-interface-id".into(),
                port: 4000,
                bundle: "test-bundle-id".into(),
                dna_interface: None,
            }),
            Ok(())
        );

        assert_eq!(
            conductor.start_ui_interface(&"test-ui-interface-id".to_string()),
            Ok(())
        );
    }

    #[test]
    fn test_stop_ui_interface() {
        let test_name = "test_stop_ui_interface";
        let mut conductor = create_test_conductor(test_name, 3005);

        let bundle_path = PathBuf::from(".");
        assert_eq!(
            conductor.install_ui_bundle_from_file(
                bundle_path,
                &"test-bundle-id".to_string(),
                false
            ),
            Ok(())
        );

        assert_eq!(
            conductor.add_ui_interface(UiInterfaceConfiguration {
                id: "test-ui-interface-id".into(),
                port: 4001,
                bundle: "test-bundle-id".into(),
                dna_interface: None,
            }),
            Ok(())
        );

        assert_eq!(
            conductor.stop_ui_interface(&"test-ui-interface-id".to_string()),
            Err(HolochainInstanceError::InternalFailure(
                HolochainError::ErrorGeneric("server is already stopped".into())
            ))
        );

        assert_eq!(
            conductor.start_ui_interface(&"test-ui-interface-id".to_string()),
            Ok(())
        );

        assert_eq!(
            conductor.stop_ui_interface(&"test-ui-interface-id".to_string()),
            Ok(())
        );
    }
}
