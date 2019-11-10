use crate::{
    nucleus::{actions::initialize::Initialization, validation::ValidationResult, ZomeFnCall},
    scheduled_jobs::pending_validations::{PendingValidation, ValidatingWorkflow},
};
use holochain_core_types::{dna::Dna, error::HolochainError, validation::ValidationPackage};

use crate::state::StateWrapper;
use holochain_json_api::{
    error::{JsonError, JsonResult},
    json::JsonString,
};
use holochain_persistence_api::cas::content::{Address, AddressableContent, Content};
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use snowflake;
use std::{collections::HashMap, convert::TryFrom, fmt};
use std::collections::{VecDeque, HashSet};

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PendingValidationKey {
    pub address: Address,
    pub workflow: ValidatingWorkflow,
}

impl Serialize for PendingValidationKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let workflow_string: String = self.workflow.to_owned().into();
        serializer.serialize_str(&format!("{}__{}", self.address, workflow_string))
    }
}

struct PendingValidationKeyStringVisitor;
impl<'de> Visitor<'de> for PendingValidationKeyStringVisitor {
    type Value = PendingValidationKey;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a PendingValidtionKey in the format '<address>__<workflow>'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let parts: Vec<String> = value.split("__").map(|s| s.to_string()).collect();
        let address = parts
            .first()
            .ok_or_else(|| Error::custom("No address found"))?
            .to_owned();
        let workflow = parts
            .last()
            .ok_or_else(|| Error::custom("No workflow found"))?
            .to_owned();
        Ok(PendingValidationKey::new(
            address.into(),
            ValidatingWorkflow::try_from(workflow).map_err(|e| Error::custom(e.to_string()))?,
        ))
    }
}

impl<'de> Deserialize<'de> for PendingValidationKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PendingValidationKeyStringVisitor)
    }
}

impl PendingValidationKey {
    pub fn new(address: Address, workflow: ValidatingWorkflow) -> Self {
        PendingValidationKey { address, workflow }
    }
}

/// The state-slice for the Nucleus.
/// Holds the dynamic parts of the DNA, i.e. zome calls and validation requests.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NucleusState {
    // Persisted fields:
    pub status: NucleusStatus,
    pub pending_validations: HashMap<PendingValidationKey, PendingValidation>,

    // Transient fields:
    pub dna: Option<Dna>, //DNA is transient here because it is stored in the chain and gets
    //read from there when loading an instance/chain.

    pub queued_zome_calls: VecDeque<ZomeFnCall>,
    pub running_zome_calls: HashSet<ZomeFnCall>,

    // @TODO eventually drop stale calls
    // @see https://github.com/holochain/holochain-rust/issues/166
    // @TODO should this use the standard ActionWrapper/ActionResponse format?
    // @see https://github.com/holochain/holochain-rust/issues/196
    pub zome_call_results: HashMap<ZomeFnCall, Result<JsonString, HolochainError>>,
    pub validation_results: HashMap<(snowflake::ProcessUniqueId, Address), ValidationResult>,
    pub validation_packages:
        HashMap<snowflake::ProcessUniqueId, Result<ValidationPackage, HolochainError>>,
}

impl NucleusState {
    pub fn new() -> Self {
        NucleusState {
            dna: None,
            status: NucleusStatus::New,
            queued_zome_calls: VecDeque::new(),
            running_zome_calls: HashSet::new(),
            zome_call_results: HashMap::new(),
            validation_results: HashMap::new(),
            validation_packages: HashMap::new(),
            pending_validations: HashMap::new(),
        }
    }

    pub fn zome_call_result(
        &self,
        zome_call: &ZomeFnCall,
    ) -> Option<Result<JsonString, HolochainError>> {
        self.zome_call_results
            .get(zome_call)
            .cloned()
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
    pub pending_validations: HashMap<PendingValidationKey, PendingValidation>,
}

impl From<&StateWrapper> for NucleusStateSnapshot {
    fn from(state: &StateWrapper) -> Self {
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
            queued_zome_calls: VecDeque::new(),
            running_zome_calls: HashSet::new(),
            zome_call_results: HashMap::new(),
            validation_results: HashMap::new(),
            validation_packages: HashMap::new(),
            pending_validations: snapshot.pending_validations,
        }
    }
}

pub static NUCLEUS_SNAPSHOT_ADDRESS: &str = "NucleusState";
impl AddressableContent for NucleusStateSnapshot {
    fn address(&self) -> Address {
        NUCLEUS_SNAPSHOT_ADDRESS.into()
    }

    fn content(&self) -> Content {
        self.to_owned().into()
    }

    fn try_from_content(content: &Content) -> JsonResult<Self> {
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
