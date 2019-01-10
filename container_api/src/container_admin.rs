use crate::container::Container;
use holochain_core_types::error::HolochainError;
use std::{
    path::PathBuf,
    sync::Arc,
};

trait ContainerAdmin {
    fn install_dna_from_file(&mut self, path: PathBuf, id: String) -> Result<(), HolochainError>;
    fn uninstall_dna(&mut self, id: String) -> Result<(), HolochainError>;
}

impl ContainerAdmin for Container {
    fn install_dna_from_file(&mut self, path: PathBuf, _id: String) -> Result<(), HolochainError> {
        let path_string = path.to_str().ok_or(HolochainError::ConfigError("invalid path".into()))?;

        let _dna = Arc::get_mut(&mut self.dna_loader).unwrap()(&path_string.into()).map_err(
            |_| {
                HolochainError::ConfigError(format!(
                    "Could not load DNA file \"{}\"",
                    path_string
                ))
            },
        )?;

        // write this file back in the managed location

        // add the new dna entry to the configuration

        // write the config file
        Ok(())
    }

    fn uninstall_dna(&mut self, _id: String) -> Result<(), HolochainError> {
        Ok(())
    }
}