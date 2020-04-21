use crate::conductor::Conductor;
use holochain_core_types::{dna::fn_declarations::FnDeclaration, error::HolochainError};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ZomePath {
    pub instance_id: String,
    pub zome_name: String,
}

pub trait ConductorIntrospection {
    fn get_zomes_by_trait(
        &mut self,
        trait_name: String,
        trait_functions: Vec<FnDeclaration>,
    ) -> Result<Vec<ZomePath>, HolochainError>;
}

impl ConductorIntrospection for Conductor {
    fn get_zomes_by_trait(
        &mut self,
        trait_name: String,
        trait_functions: Vec<FnDeclaration>,
    ) -> Result<Vec<ZomePath>, HolochainError> {
        let _ = trait_name;
        let _ = trait_functions;
        Ok(Vec::new())
    }
}
