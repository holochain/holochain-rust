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
    fn remove_interface(&mut self, id: &String) -> Result<(), HolochainError>;
    
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
    /// Also stops then removes its UI interface
    fn uninstall_ui_bundle(&mut self, id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        new_config.ui_bundles = new_config
            .ui_bundles
            .into_iter()
            .filter(|bundle| bundle.id != *id)
            .collect();

        // TODO: add logic to stop and remove the UI interface if it has one

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

    fn remove_interface(&mut self, id: &String) -> Result<(), HolochainError> {
        let mut new_config = self.config.clone();
        new_config.ui_interfaces = new_config
            .ui_interfaces
            .into_iter()
            .filter(|ui_interface| ui_interface.id != *id)
            .collect();

        // TODO: Also remove any references in UI bundles

        new_config.check_consistency()?;
        self.config = new_config;
        self.save_config()?;
        Ok(())    
    }

    fn start_ui_interface(&mut self, id: &String) -> Result<(), HolochainInstanceError> {
        let _server = self.static_servers.get(id)?;
        notify(format!("Starting UI interface \"{}\"...", id));
        // server.start()
        Ok(())
    }

    fn stop_ui_interface(&mut self, id: &String) -> Result<(), HolochainInstanceError> {
        let _server = self.static_servers.get(id)?;
        notify(format!("Stopping UI interface \"{}\"...", id));
        // server.stop()
        Ok(())
    }
}
