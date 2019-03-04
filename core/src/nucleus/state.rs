use crate::{
    nucleus::{actions::initialize::Initialization, validation::ValidationResult, ZomeFnCall},
    scheduled_jobs::pending_validations::{PendingValidation, ValidatingWorkflow},
    state::State,
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent, Content},
    dna::Dna,
    error::HolochainError,
    json::JsonString,
    validation::ValidationPackage,
};
use snowflake;
use std::{collections::HashMap, convert::TryFrom};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, DefaultJson)]
pub enum NucleusStatus {
    New,
    Initializing,
    Initialized(Initialization),
    InitializationFailed(String),
}

impl Default for NucleusStatus {
    fn default() -> Self {
        NucleusStatus::New
    }
}

/// The state-slice for the Nucleus.
/// Holds the dynamic parts of the DNA, i.e. zome calls and validation requests.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NucleusState {
    // Persisted fields:
    pub status: NucleusStatus,
    pub pending_validations: HashMap<(Address, ValidatingWorkflow), PendingValidation>,

    // Transient fields:
    pub dna: Option<Dna>, //DNA is transient here because it is stored in the chain and gets
    //read from there when loading an instance/chain.

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
            pending_validations: HashMap::new(),
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
        match self.status {
            NucleusStatus::Initialized(_) => true,
            _ => false,
        }
    }

    pub fn initialization(&self) -> Option<Initialization> {
        match self.status {
            NucleusStatus::Initialized(ref init) => Some(init.clone()),
            _ => None,
        }
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

#[derive(Clone, Debug, Deserialize, Serialize, DefaultJson)]
pub struct NucleusStateSnapshot {
    pub status: NucleusStatus,
    pub pending_validations: HashMap<(Address, ValidatingWorkflow), PendingValidation>,
}

impl From<&State> for NucleusStateSnapshot {
    fn from(state: &State) -> Self {
        NucleusStateSnapshot {
            status: state.nucleus().status(),
            pending_validations: state.nucleus().pending_validations.clone(),
        }
    }
}

impl From<NucleusStateSnapshot> for NucleusState {
    fn from(snapshot: NucleusStateSnapshot) -> Self {
        NucleusState {
            dna: None,
            status: snapshot.status,
            zome_calls: HashMap::new(),
            validation_results: HashMap::new(),
            validation_packages: HashMap::new(),
            pending_validations: snapshot.pending_validations,
        }
    }
}

pub static NUCLEUS_SNAPSHOT_ADDRESS: &'static str = "NucleusState";
impl AddressableContent for NucleusStateSnapshot {
    fn address(&self) -> Address {
        NUCLEUS_SNAPSHOT_ADDRESS.into()
    }

    fn content(&self) -> Content {
        self.to_owned().into()
    }

    fn try_from_content(content: &Content) -> Result<Self, HolochainError> {
        Self::try_from(content.to_owned())
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
