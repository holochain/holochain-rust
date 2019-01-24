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
    
    fn start_ui_interface(&mut self, id: &String) -> Result<(), HolochainError>;
    fn stop_ui_interface(&mut self, id: &String) -> Result<(), HolochainError>;
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

    fn uninstall_ui_bundle(&mut self, id: &String) -> Result<(), HolochainError> {
        Ok(())
    }
 
    fn add_ui_interface(&mut self, new_instance: UiInterfaceConfiguration) -> Result<(), HolochainError> {
        Ok(())
    }

    fn remove_interface(&mut self, id: &String) -> Result<(), HolochainError> {
        Ok(())
    }

    fn start_ui_interface(&mut self, id: &String) -> Result<(), HolochainError> {
        Ok(())
    }

    fn stop_ui_interface(&mut self, id: &String) -> Result<(), HolochainError> {
        Ok(())
    }
}
