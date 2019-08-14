use crate::{
    conductor::{Conductor},
};
use serde::Serialize;
use holochain_core_types::error::HolochainError;
use holochain_persistence_api::reporting::StorageReport;

#[derive(Serialize)]
pub struct InstanceStorageReport {
	pub chain: StorageReport,
	pub dht: StorageReport,
	pub eav: StorageReport,
}

impl InstanceStorageReport {
	pub fn new(chain: StorageReport, dht: StorageReport, eav: StorageReport) -> Self {
		Self {
			chain,
			dht,
			eav,
		}
	}
}

pub trait ConductorStatInterface {
	fn get_instance_storage(&self, instance_id: &String) -> Result<InstanceStorageReport, HolochainError>;
}

impl ConductorStatInterface for Conductor {
	fn get_instance_storage(&self, instance_id: &String) -> Result<InstanceStorageReport, HolochainError> {
		let instance = self.instances.get(instance_id)?.read()?;
		Ok(
			InstanceStorageReport::new(
				instance.context()?.chain_storage.read()?.get_storage_report()?.clone(),
				instance.context()?.dht_storage.read()?.get_storage_report()?.clone(),
				instance.context()?.eav_storage.read()?.get_storage_report()?.clone(),
			)
		)
	}
}
