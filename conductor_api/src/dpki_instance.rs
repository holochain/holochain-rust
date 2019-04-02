/// Defines a rust wrapper trait to match the Dpki exposed function trait definition
/// for the conductor to use in the various dpki related flows, e.g. bootstrap and agent addition
use crate::holochain::Holochain;
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    error::HolochainError,
};

use holochain_core::{
    context::Context,
    nucleus::{
        actions::call_zome_function::make_cap_request_for_call,
        ribosome::capabilities::CapabilityRequest,
    },
};

pub const DPKI_ZOME_NAME: &str = "dpki";
pub const DPKI_TRAIT_FN_ADD_AGENT: &str = "create_agent_key";
pub const DPKI_TRAIT_FN_INIT: &str = "init";
pub const DPKI_TRAIT_FN_IS_INITIALIZED: &str = "is_initialized";

use std::{convert::TryInto, sync::Arc};

pub trait DpkiInstance {
    fn dpki_create_agent_key(&mut self, agent_name: String) -> Result<(), HolochainError>;
    fn dpki_init(&mut self, params: String) -> Result<(), HolochainError>;
    fn dpki_is_initialized(&mut self) -> Result<bool, HolochainError>;
}

/// create a capability request for a given dpki call
fn dpki_cap_request(
    context: Arc<Context>,
    function: &str,
    parameters: &str,
) -> Result<CapabilityRequest, HolochainError> {
    let token = Address::from(context.agent_id.address());
    Ok(make_cap_request_for_call(
        context.clone(),
        token,
        function,
        parameters.to_string(),
    ))
}

impl DpkiInstance for Holochain {
    /// wrapper for the dpki create_agent_key trait function
    fn dpki_create_agent_key(&mut self, agent_name: String) -> Result<(), HolochainError> {
        let params = json!({ "agent_name": agent_name }).to_string();
        let cap_request =
            dpki_cap_request(self.context().clone(), DPKI_TRAIT_FN_ADD_AGENT, &params)?;
        let _result = self.call(
            DPKI_ZOME_NAME,
            cap_request,
            DPKI_TRAIT_FN_ADD_AGENT,
            &params,
        )?;
        Ok(())
    }

    // wrapper for the dpki init trait function
    fn dpki_init(&mut self, params: String) -> Result<(), HolochainError> {
        let params = json!({ "params": params }).to_string();
        let cap_request = dpki_cap_request(self.context().clone(), DPKI_TRAIT_FN_INIT, &params)?;
        let _result = self.call(DPKI_ZOME_NAME, cap_request, DPKI_TRAIT_FN_INIT, &params)?;
        Ok(())
    }

    // wrapper for the dpki is_initialized trait function
    fn dpki_is_initialized(&mut self) -> Result<bool, HolochainError> {
        let params = "{}";
        let cap_request =
            dpki_cap_request(self.context().clone(), DPKI_TRAIT_FN_IS_INITIALIZED, params)?;
        let result = self.call(
            DPKI_ZOME_NAME,
            cap_request,
            DPKI_TRAIT_FN_IS_INITIALIZED,
            params,
        )?;
        let result: bool = result.try_into()?;
        Ok(result)
    }
}
