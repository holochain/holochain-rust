use crate::nucleus::ZomeFnCall;
use holochain_core_types::{
    cas::content::Address, dna::Dna, error::HolochainError, json::JsonString,
    validation::ValidationPackage,
};
use snowflake;
use std::collections::HashMap;
#[derive(Clone, Debug, PartialEq)]
pub enum NucleusStatus {
    New,
    Initializing,
    Initialized,
    InitializationFailed(String),
}

impl Default for NucleusStatus {
    fn default() -> Self {
        NucleusStatus::New
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValidationError {
    Fail(String),
    UnresolvedDependencies(Vec<Address>),
    NotImplemented,
    Error(String),
}
pub type ValidationResult = Result<(), ValidationError>;

impl From<ValidationError> for HolochainError {
    fn from(ve: ValidationError) -> Self {
        match ve {
            ValidationError::Fail(reason) => HolochainError::ValidationFailed(reason),
            ValidationError::UnresolvedDependencies(_) => HolochainError::ValidationFailed("Missing dependencies".to_string()),
            ValidationError::NotImplemented => HolochainError::NotImplemented("Validation not implemented".to_string()),
            ValidationError::Error(e) => HolochainError::ErrorGeneric(e),
        }
    }
}

/// The state-slice for the Nucleus.
/// Holds the dynamic parts of the DNA, i.e. zome calls and validation requests.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NucleusState {
    pub dna: Option<Dna>,
    pub status: NucleusStatus,
    // @TODO eventually drop stale calls
    // @see https://github.com/holochain/holochain-rust/issues/166
    // @TODO should this use the standard ActionWrapper/ActionResponse format?
    // @see https://github.com/holochain/holochain-rust/issues/196
    pub zome_calls: HashMap<ZomeFnCall, Option<Result<JsonString, HolochainError>>>,
    pub validation_results: HashMap<(snowflake::ProcessUniqueId, Address), ValidationResult>,
    pub validation_packages:
        HashMap<snowflake::ProcessUniqueId, Result<ValidationPackage, HolochainError>>,
}

impl NucleusState {
    pub fn new() -> Self {
        NucleusState {
            dna: None,
            status: NucleusStatus::New,
            zome_calls: HashMap::new(),
            validation_results: HashMap::new(),
            validation_packages: HashMap::new(),
        }
    }

    pub fn zome_call_result(
        &self,
        zome_call: &ZomeFnCall,
    ) -> Option<Result<JsonString, HolochainError>> {
        self.zome_calls
            .get(zome_call)
            .and_then(|value| value.clone())
    }

    pub fn has_initialized(&self) -> bool {
        self.status == NucleusStatus::Initialized
    }

    pub fn has_initialization_failed(&self) -> bool {
        match self.status {
            NucleusStatus::InitializationFailed(_) => true,
            _ => false,
        }
    }

    // Getters
    pub fn dna(&self) -> Option<Dna> {
        self.dna.clone()
    }
    pub fn status(&self) -> NucleusStatus {
        self.status.clone()
    }
}

#[cfg(test)]
pub mod tests {

    use super::NucleusState;

    /// dummy nucleus state
    pub fn test_nucleus_state() -> NucleusState {
        NucleusState::new()
    }

}
