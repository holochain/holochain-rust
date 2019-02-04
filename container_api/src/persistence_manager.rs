use holochain_core_types::{
	error::HcResult,
	dna::Dna,
};
use crate::config::Configuration;
use std::path::PathBuf;

pub enum Location {
	FilePath(PathBuf)
}

pub trait PersistenceManager {
	fn load_config(&self, location: Location) -> HcResult<Configuration>;
	fn save_config(&self, location: Location) -> HcResult<()>;
	fn load_dna(&self, location: Location) -> HcResult<Dna>;
	fn save_dna(&self, location: Location) -> HcResult<()>;
	fn copy_ui_dir(&self, source: Location, dest: Location) -> HcResult<()>;
}
