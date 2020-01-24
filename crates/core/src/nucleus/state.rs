use crate::{
    dht::pending_validations::ValidatingWorkflow,
    nucleus::{
        actions::initialize::Initialization, WasmApiFnCall, WasmApiFnCallResult, ZomeFnCall,
    },
};
use holochain_core_types::{dna::Dna, error::HolochainError};

use crate::state::StateWrapper;
use holochain_json_api::{
    error::{JsonError, JsonResult},
    json::JsonString,
};
use holochain_persistence_api::cas::content::{Address, AddressableContent, Content};
use im::{HashMap, HashSet};
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::VecDeque, convert::TryFrom, fmt};

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

    // Transient fields:
    pub dna: Option<Dna>, //DNA is transient here because it is stored in the chain and gets
    //read from there when loading an instance/chain.
    pub queued_zome_calls: VecDeque<ZomeFnCall>,
    pub running_zome_calls: HashSet<ZomeFnCall>,
    pub wasm_api_function_calls: HashMap<ZomeFnCall, ZomeFnCallState>,
    pub zome_call_results: HashMap<ZomeFnCall, Result<JsonString, HolochainError>>,
}

impl NucleusState {
    pub fn new() -> Self {
        NucleusState {
            dna: None,
            status: NucleusStatus::New,
            queued_zome_calls: VecDeque::new(),
            running_zome_calls: HashSet::new(),
            zome_call_results: HashMap::new(),
            wasm_api_function_calls: HashMap::new(),
        }
    }

    pub fn zome_call_result(
        &self,
        zome_call: &ZomeFnCall,
    ) -> Option<Result<JsonString, HolochainError>> {
        self.zome_call_results.get(zome_call).cloned()
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
}

impl From<&StateWrapper> for NucleusStateSnapshot {
    fn from(state: &StateWrapper) -> Self {
        NucleusStateSnapshot {
            status: state.nucleus().status(),
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
            wasm_api_function_calls: HashMap::new(),
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

#[derive(Clone, Default, Debug, PartialEq, Serialize)]
pub struct ZomeFnCallState {
    wasm_api_fn_invocations: Vec<(WasmApiFnCall, Option<WasmApiFnCallResult>)>,
}

impl ZomeFnCallState {
    pub fn begin_wasm_api_call(&mut self, call: WasmApiFnCall) {
        self.wasm_api_fn_invocations.push((call, None))
    }

    pub fn end_wasm_api_call(
        &mut self,
        call: WasmApiFnCall,
        result: WasmApiFnCallResult,
    ) -> Result<(), HolochainError> {
        if let Some((current_call, current_result)) = self.wasm_api_fn_invocations.pop() {
            if call != current_call {
                Err(HolochainError::new(
                    "Wasm API call other than the current call was ended.",
                ))
            } else if current_result.is_some() {
                Err(HolochainError::new(
                    "Ending and Wasm API which was already ended.",
                ))
            } else {
                self.wasm_api_fn_invocations.push((call, Some(result)));
                Ok(())
            }
        } else {
            Err(HolochainError::new(
                "Attempted to end Wasm API call, but none was started!",
            ))
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::{WasmApiFnCall, NucleusState, ZomeFnCallState};
    use crate::wasm_engine::api::ZomeApiFunction;

    /// dummy nucleus state
    pub fn test_nucleus_state() -> NucleusState {
        NucleusState::new()
    }

    #[test]
    fn test_zome_fn_call_state() {
        let mut state = ZomeFnCallState::default();
        let call1 = WasmApiFnCall {
            function: ZomeApiFunction::Call,
            parameters: "params1".into(),
        };
        let call2 = WasmApiFnCall {
            function: ZomeApiFunction::Call,
            parameters: "params2".into(),
        };

        state.begin_wasm_api_call(call1.clone());
        state.end_wasm_api_call(call1, Ok("result".into())).unwrap();

        state.begin_wasm_api_call(call2.clone());
        state
            .end_wasm_api_call(call2, Err("call failed for reasons".into()))
            .unwrap();

        assert_eq!(state.wasm_api_fn_invocations.len(), 2);
    }
}
