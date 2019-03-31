/// Defines a rust wrapper trait to match the Dpki exposed function trait definition
/// for the conductor to use in the various dpki related flows, e.g. bootstrap and agent addition
use crate::holochain::Holochain;
use holochain_core_types::{
    cas::content::Address,
    error::HolochainError,
    signature::{Provenance, Signature},
};

use holochain_core::nucleus::{
    actions::call_zome_function::make_cap_request_for_call,
    ribosome::capabilities::CapabilityRequest,
};

pub const DPKI_ZOME_NAME: &str = "dpki";
pub const DPKI_TRAIT_FN_ADD_AGENT: &str = "create_agent_key";

pub trait DpkiInstance {
    fn dpki_cap_request(
        &mut self,
        method: &str,
        params: &str,
    ) -> Result<CapabilityRequest, HolochainError>;
    fn dpki_create_agent_key(&mut self, agent_name: String) -> Result<(), HolochainError>;
}

impl DpkiInstance for Holochain {
    /// create a capability request for a given dpki call
    fn dpki_cap_request(
        &mut self,
        method: &str,
        params: &str,
    ) -> Result<CapabilityRequest, HolochainError> {
        let token = Address::from("");
        let provenance = Provenance::new(Address::from(""), Signature::fake());
        Ok(CapabilityRequest::new(
            token,
            provenance.source(),
            provenance.signature(),
        ))
    }

    fn dpki_create_agent_key(&mut self, agent_name: String) -> Result<(), HolochainError> {
        let params = json!({ "agent_name": agent_name }).to_string();
        let cap_request = self.dpki_cap_request(DPKI_TRAIT_FN_ADD_AGENT, &params)?;
        let result = self.call(
            DPKI_ZOME_NAME,
            cap_request,
            DPKI_TRAIT_FN_ADD_AGENT,
            &params,
        )?;
        Ok(())
    }
}
