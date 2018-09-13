use error::HolochainError;
use holochain_dna::{wasm::DnaWasm, zome::capabilities::Capability, Dna};
use nucleus::{ZomeFnCall, ZomeFnResult};
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
    pub zome_calls: HashMap<ZomeFnCall, Option<Result<String, HolochainError>>>,
}

impl NucleusState {
    pub fn new() -> Self {
        NucleusState {
            dna: None,
            status: NucleusStatus::New,
            zome_calls: HashMap::new(),
        }
    }

    pub fn zome_call_result(
        &self,
        zome_call: &ZomeFnCall,
    ) -> Option<Result<String, HolochainError>> {
        match self.zome_calls.get(zome_call) {
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

    // Return Capability for ZomeFnCall request
    pub fn get_capability(&self, zome_call: &ZomeFnCall) -> Result<Capability, ZomeFnResult> {
        // Must have DNA
        let dna = self.dna.as_ref();
        if dna.is_none() {
            return Err(ZomeFnResult::new(zome_call.clone(), Err(HolochainError::DnaMissing)));
        }
        let dna = dna.unwrap();
        // Get Capability from DNA
        let res = dna.get_capability_with_zome_name(&zome_call.zome_name, &zome_call.cap_name);
        if let Err(e) = res {
            return Err(ZomeFnResult::new(
                zome_call.clone(),
                Err(HolochainError::DnaError(e)),
            ));
        }
        Ok(res.unwrap().clone())
    }


    // Return WASM for ZomeFnCall request
    pub fn get_fn_wasm(&self, zome_call: &ZomeFnCall) -> Result<DnaWasm, ZomeFnResult> {
        let res = self.get_capability(zome_call);
        if res.is_err() {
            return Err(res.err().unwrap());
        }
        Ok(res.unwrap().code)
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
