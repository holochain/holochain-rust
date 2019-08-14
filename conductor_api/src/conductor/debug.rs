use conductor::Conductor;
use holochain_core_types::error::HolochainError;
use holochain_core::state_dump::StateDump;

pub trait ConductorDebug {
    fn running_instances(&self) -> Result<Vec<String>, HolochainError>;
    fn state_dump_for_instance(&self, instance_id: &String)
        -> Result<StateDump, HolochainError>;
}

impl ConductorDebug for Conductor {
    fn running_instances(&self) -> Result<Vec<String>, HolochainError> {
        Ok(self.instances.keys().cloned().collect())
    }

    fn state_dump_for_instance(&self, instance_id: &String)
        -> Result<StateDump, HolochainError> {
        let hc = self.instances.get(instance_id)?;
        Ok(hc.read().unwrap().get_state_dump()?)
    }
}