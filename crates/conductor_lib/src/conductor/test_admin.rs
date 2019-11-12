use crate::{
    conductor::{base::notify, Conductor},
    config::AgentConfiguration,
    key_loaders::test_keystore,
    keystore::PRIMARY_KEYBUNDLE_ID,
};
use holochain_core_types::error::HolochainError;

pub trait ConductorTestAdmin {
    fn add_test_agent(&mut self, id: String, name: String) -> Result<String, HolochainError>;
}

impl ConductorTestAdmin for Conductor {
    fn add_test_agent(&mut self, id: String, name: String) -> Result<String, HolochainError> {
        let mut new_config = self.config.clone();
        if new_config.agents.iter().any(|i| i.id == id) {
            return Err(HolochainError::ErrorGeneric(format!(
                "Agent with ID '{}' already exists",
                id
            )));
        }
        let mut keystore = test_keystore(&name);
        let keybundle = keystore
            .get_keybundle(PRIMARY_KEYBUNDLE_ID)
            .expect("Couldn't get KeyBundle that was just added back from Keystore");
        let public_address = keybundle.get_id();
        let new_agent = AgentConfiguration {
            id: id.clone(),
            name: name.clone(),
            public_address: public_address.clone(),
            keystore_file: name.clone().into(),
            holo_remote_key: None,
            test_agent: Some(true),
        };
        new_config.agents.push(new_agent);
        new_config.check_consistency(&mut self.dna_loader)?;
        self.config = new_config;
        self.add_agent_keystore(id.clone(), keystore);
        self.save_config()?;
        notify(format!("Added agent \"{}\"", id));
        Ok(public_address)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::conductor::admin::tests::create_test_conductor;

    #[test]
    fn test_add_test_agent() {
        let test_name = "test_add_test_agent";
        let agent_id = "testAgent1".to_string();
        let mut conductor = create_test_conductor(test_name, 5001);
        let agent_address = conductor
            .add_test_agent(agent_id.clone(), "Test Agent 1".to_string())
            .expect("Could not add test agent");
        assert_eq!(agent_address.len(), 63,);
        assert!(conductor.get_keystore_for_agent(&agent_id).is_ok());
    }
}
