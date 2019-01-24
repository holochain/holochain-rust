use crate::{
    config::{
        UiInterfaceConfiguration,
    },
    container::{Container},
};
use holochain_core_types::{error::HolochainError};
use std::{path::PathBuf};

pub trait ContainerUiAdmin {
    fn install_ui_bundle_from_file(&mut self, path: PathBuf, id: String) -> Result<(), HolochainError>;
    fn uninstall_ui_bundle(&mut self, id: &String) -> Result<(), HolochainError>;

    fn add_ui_interface(&mut self, new_instance: UiInterfaceConfiguration)
        -> Result<(), HolochainError>;
    fn remove_interface(&mut self, id: &String) -> Result<(), HolochainError>;
    
    fn start_ui_interface(&mut self, id: &String) -> Result<(), HolochainError>;
    fn stop_ui_interface(&mut self, id: &String) -> Result<(), HolochainError>;
}

impl ContainerUiAdmin for Container {
    fn install_ui_bundle_from_file(&mut self, path: PathBuf, id: String) -> Result<(), HolochainError> {
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
