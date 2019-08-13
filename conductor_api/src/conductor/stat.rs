use crate::{
    conductor::{Conductor},
};
use holochain_core_types::error::HolochainError;


pub trait ConductorStatInterface {
	fn get_instance_storage(&self, instance_id: &String) -> Result<usize, HolochainError>;
}

impl ConductorStatInterface for Conductor {
	/// Return the number of bytes currently being used by an instance for local storage (CAS-chain + CAS-DHT + EAV)
	fn get_instance_storage(&self, instance_id: &String) -> Result<usize, HolochainError> {
		let instance = self.instances.get(instance_id)?.read()?;
		let context = instance.context()?;
		let chain_storage_bytes = context.chain_storage.read()?.get_byte_count()?;
		let dht_storage_bytes = context.dht_storage.read()?.get_byte_count()?;
		let eav_storage_bytes = context.eav_storage.read()?.get_byte_count()?;
		Ok(chain_storage_bytes + dht_storage_bytes + eav_storage_bytes)
	}
}
