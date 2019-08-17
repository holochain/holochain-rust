use crate::conductor::Conductor;
use holochain_core_types::error::HolochainError;
use holochain_persistence_api::reporting::StorageReport;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct InstanceStorageReport {
    pub chain: StorageReport,
    pub dht: StorageReport,
    pub eav: StorageReport,
}

impl InstanceStorageReport {
    pub fn new(chain: StorageReport, dht: StorageReport, eav: StorageReport) -> Self {
        Self { chain, dht, eav }
    }
}

pub trait ConductorStatInterface {
    fn get_instance_storage(
        &self,
        instance_id: &String,
    ) -> Result<InstanceStorageReport, HolochainError>;
}

impl ConductorStatInterface for Conductor {
    fn get_instance_storage(
        &self,
        instance_id: &String,
    ) -> Result<InstanceStorageReport, HolochainError> {
        let instance = self.instances.get(instance_id)?.read()?;
        Ok(InstanceStorageReport::new(
            instance
                .context()?
                .chain_storage
                .read()?
                .get_storage_report()?
                .clone(),
            instance
                .context()?
                .dht_storage
                .read()?
                .get_storage_report()?
                .clone(),
            instance
                .context()?
                .eav_storage
                .read()?
                .get_storage_report()?
                .clone(),
        ))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::conductor::admin::tests::create_test_conductor;

    #[test]
    fn test_call_get_storage() {
        let test_name = "test_call_get_storage";
        let conductor = create_test_conductor(test_name, 7771);
        assert_eq!(
            conductor.get_instance_storage(&"test-instance-1".to_string()),
            Err(HolochainError::ErrorGeneric(
                "Not implemented for this storage type".into()
            )),
        )
    }
}
