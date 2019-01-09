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
    fn install_dna_from_file(&mut self, path: PathBuf, id: String) -> Result<(), HolochainError> {
        let dna = Arc::get_mut(&mut self.dna_loader).unwrap()(path.to_str()).map_err(
            |_| {
                HolochainError::ConfigError(format!(
                    "Could not load DNA file \"{}\"",
                    path.to_str()
                ))
            },
        )?;
    }

    fn uninstall_dna(&mut self, id: String) -> Result<(), HolochainError> {

    }
}