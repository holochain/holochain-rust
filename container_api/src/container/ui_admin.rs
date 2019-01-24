use error::HolochainInstanceError;
use crate::{
    config::{
        UiInterfaceConfiguration,
        UiBundleConfiguration,
    },
    container::{Container, base::notify},
};
use holochain_core_types::{error::HolochainError};
use std::{path::PathBuf};

pub trait ContainerUiAdmin {
    fn install_ui_bundle_from_file(&mut self, path: PathBuf, id: &String) -> Result<(), HolochainError>;
    fn uninstall_ui_bundle(&mut self, id: &String) -> Result<(), HolochainError>;

    fn add_ui_interface(&mut self, new_instance: UiInterfaceConfiguration)
        -> Result<(), HolochainError>;
    fn remove_ui_interface(&mut self, id: &String) -> Result<(), HolochainError>;
    
    fn start_ui_interface(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
    fn stop_ui_interface(&mut self, id: &String) -> Result<(), HolochainInstanceError>;
}

impl ContainerUiAdmin for Container {
    fn install_ui_bundle_from_file(&mut self, path: PathBuf, id: &String) -> Result<(), HolochainError> {
        let path_string = path
            .to_str()
            .ok_or(HolochainError::ConfigError("invalid path".into()))?;

        let new_bundle = UiBundleConfiguration {
            id: id.to_string(),
            root_dir: path_string.into(),
            hash: "1234".to_string(),
        };

        let mut new_config = self.config.clone();
        new_config.ui_bundles.push(new_bundle.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        notify(format!("Installed UI bundle from {} as \"{}\"", path_string, id));
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

        let to_remove = new_config.ui_interfaces
            .into_iter()
            .filter(|ui_interface| ui_interface.bundle == id.to_string());

        for bundle_interface in to_remove {
            self.remove_ui_interface(&bundle_interface.id)?;
        }

        Ok(())
    }
 
    fn add_ui_interface(&mut self, new_interface: UiInterfaceConfiguration) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        new_config.ui_interfaces.push(new_interface.clone());
        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        Ok(())
    }

    fn remove_ui_interface(&mut self, id: &String) -> Result<(), HolochainError> {

        let to_stop = self.config.clone().ui_interfaces
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
    use container::admin::tests::*;

    #[test]
    fn test_install_ui_bundle_from_file() {
        let mut container = create_test_container("test_install_dna_from_file");
        let bundle_path = PathBuf::from(".");
        container.install_ui_bundle_from_file(bundle_path, &"test-bundle-id".to_string())
            .unwrap()
    }

    #[test]
    fn test_uninstall_ui_bundle() {
        let mut container = create_test_container("test_install_dna_from_file");
        container.uninstall_ui_bundle(&"test-bundle-id".to_string())
            .unwrap()
    }

    #[test]
    fn test_add_ui_interface() {
        let mut container = create_test_container("test_install_dna_from_file");
        container.add_ui_interface(UiInterfaceConfiguration {
            id: "test-ui-interface-id".into(),
            port: 3000,
            bundle: "test-bundle_id".into(),
            dna_interface: None,
        }).unwrap()
    }

    #[test]
    fn test_remove_ui_interface() {
        let mut container = create_test_container("test_install_dna_from_file");
        container.remove_ui_interface(&"test-ui-interface-id".to_string())
            .unwrap()
    }

    #[test]
    fn test_start_ui_interface() {
        let mut container = create_test_container("test_install_dna_from_file");
        container.start_ui_interface(&"test-ui-interface-id".to_string())
            .unwrap()
    }

    #[test]
    fn test_stop_ui_interface() {
        let mut container = create_test_container("test_install_dna_from_file");
        container.stop_ui_interface(&"test-ui-interface-id".to_string())
            .unwrap()
    }
}
