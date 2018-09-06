use error::HolochainError;
use holochain_dna::{wasm::DnaWasm, zome::capabilities::Capability, Dna};
use nucleus::{FunctionCall, FunctionResult};
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

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NucleusState {
    pub dna: Option<Dna>,
    pub status: NucleusStatus,
    // @TODO eventually drop stale calls
    // @see https://github.com/holochain/holochain-rust/issues/166
    // @TODO should this use the standard ActionWrapper/ActionResponse format?
    // @see https://github.com/holochain/holochain-rust/issues/196
    pub ribosome_calls: HashMap<FunctionCall, Option<Result<String, HolochainError>>>,
}

impl NucleusState {
    pub fn new() -> Self {
        NucleusState {
            dna: None,
            status: NucleusStatus::New,
            ribosome_calls: HashMap::new(),
        }
    }

    pub fn ribosome_call_result(
        &self,
        function_call: &FunctionCall,
    ) -> Option<Result<String, HolochainError>> {
        match self.ribosome_calls.get(function_call) {
            None => None,
            Some(value) => value.clone(),
        }
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

    // Return WASM from FunctionCall request
    pub fn get_fn_wasm(&self, fc: FunctionCall) -> Result<DnaWasm, FunctionResult> {
        // Must have DNA
        let dna = self.dna.as_ref();
        if dna.is_none() {
            return Err(FunctionResult::new(fc, Err(HolochainError::DnaMissing)));
        }
        let dna = dna.unwrap();
        // Zome must exist in DNA
        let zome = dna.get_zome(&fc.zome);
        if zome.is_none() {
            return Err(FunctionResult::new(
                fc.clone(),
                Err(HolochainError::ZomeNotFound(format!(
                    "Zome '{}' not found",
                    &fc.zome
                ))),
            ));
        }
        let zome = zome.unwrap();
        // Capability must exist in Zome
        let wasm = dna.get_wasm_from_capability(zome, &fc.capability);
        if wasm.is_none() {
            return Err(FunctionResult::new(
                fc.clone(),
                Err(HolochainError::CapabilityNotFound(format!(
                    "Capability '{:?}' not found in Zome '{:?}'",
                    &fc.capability, &fc.zome
                ))),
            ));
        }
        // Everything OK
        Ok(wasm.unwrap().clone())
    }

    // Return Capability from FunctionCall request
    pub fn get_capability(&self, fc: FunctionCall) -> Result<Capability, FunctionResult> {
        // Must have DNA
        let dna = self.dna.as_ref();
        if dna.is_none() {
            return Err(FunctionResult::new(fc, Err(HolochainError::DnaMissing)));
        }
        let dna = dna.unwrap();
        // Zome must exist in DNA
        let zome = dna.get_zome(&fc.zome);
        if zome.is_none() {
            return Err(FunctionResult::new(
                fc.clone(),
                Err(HolochainError::ZomeNotFound(format!(
                    "Zome '{}' not found",
                    &fc.zome
                ))),
            ));
        }
        let zome = zome.unwrap();
        // Capability must exist in Zome
        let cap = dna.get_capability(zome, &fc.capability);
        if cap.is_none() {
            return Err(FunctionResult::new(
                fc.clone(),
                Err(HolochainError::CapabilityNotFound(format!(
                    "Capability '{:?}' not found in Zome '{:?}'",
                    &fc.capability, &fc.zome
                ))),
            ));
        }
        // Everything OK
        Ok(cap.unwrap().clone())
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
