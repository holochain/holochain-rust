extern crate futures;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{
        is_fn_public,
        ribosome,
        ZomeFnCall, ZomeFnResult,
    },
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::error::HolochainError;
use std::{pin::Pin, sync::Arc};

use holochain_core_types::json::JsonString;
use holochain_core_types::{
    dna::{capabilities::CapabilityCall},
    entry::cap_entries::CapTokenGrant,
};
use std::{convert::TryFrom, thread};

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ExecuteZomeFnResponse {
    call: ZomeFnCall,
    result: ZomeFnResult,
}

impl ExecuteZomeFnResponse {
    pub fn new(call: ZomeFnCall, result: Result<JsonString, HolochainError>) -> Self {
        ExecuteZomeFnResponse { call, result }
    }

    /// read only access to call
    pub fn call(&self) -> ZomeFnCall {
        self.call.clone()
    }

    /// read only access to result
    pub fn result(&self) -> Result<JsonString, HolochainError> {
        self.result.clone()
    }
}

/// Initialize Application, Action Creator
/// This is the high-level initialization function that wraps the whole process of initializing an
/// instance. It creates both InitApplication and ReturnInitializationResult actions asynchronously.
///
/// Returns a future that resolves to an Ok(NucleusStatus) or an Err(String) which carries either
/// the Dna error or errors from the genesis callback.
///
/// Use futures::executor::block_on to wait for an initialized instance.
pub async fn call_zome_function(
    zome_call: ZomeFnCall,
    context: &Arc<Context>,
) -> Result<JsonString, HolochainError> {
    let (dna_name, code) = {
        let state = context.state().ok_or(HolochainError::ErrorGeneric(
            "Context not initialized".to_string(),
        ))?;
        let nucleus_state = state.nucleus();
        let dna = nucleus_state
            .dna
            .as_ref()
            .ok_or(HolochainError::DnaMissing)?;

        // 1. Validate the call (a number of things could go wrong)
        // 1.a make sure the zome and function exists
        let _ = dna
            .get_function_with_zome_name(&zome_call.zome_name, &zome_call.fn_name)
            .map_err(HolochainError::Dna)?;

        // 1.b make sure caller is allowed to call the function
        let public = is_fn_public(&dna, &zome_call)?;
        if !public && !check_capability(context.clone(), &zome_call.clone()) {
            return Err(HolochainError::CapabilityCheckFailed);
        }

        let dna_name = dna.name.clone();
        let code = dna
            .get_wasm_from_zome_name(zome_call.zome_name.clone())
            .expect("zome not found, Should have failed before when getting capability.")
            .code
            .clone();

        (dna_name, code)
    };

    // 2. function WASM and execute it in a separate thread

    let context_clone = context.clone();
    let zome_call_clone = zome_call.clone();

    let _ = thread::spawn(move || {
        // Have Ribosome spin up DNA and call the zome function
        let call_result = ribosome::run_dna(
            &dna_name,
            context_clone.clone(),
            code,
            &zome_call_clone,
            Some(zome_call_clone.clone().parameters.into_bytes()),
        );
        // Construct response
        let response = ExecuteZomeFnResponse::new(zome_call_clone, call_result);
        // Send ReturnZomeFunctionResult Action
        context_clone
            .action_channel()
            .send(ActionWrapper::new(Action::ReturnZomeFunctionResult(
                response,
            )))
            .expect("action channel to be open in reducer");
    });

    await!(CallResultFuture {
        context: context.clone(),
        zome_call,
    })
}


// TODO: check the signature too
fn is_token_the_agent(context: Arc<Context>, cap: &Option<CapabilityCall>) -> bool {
    match cap {
        None => false,
        Some(call) => context.agent_id.key == call.cap_token.to_string(),
    }
}

/// checks to see if a given function call is allowable according to the capabilities
/// that have been registered to callers in the chain.
fn check_capability(context: Arc<Context>, fn_call: &ZomeFnCall) -> bool {
    // the agent can always do everything
    if is_token_the_agent(context.clone(), &fn_call.cap) {
        return true;
    }

    match fn_call.cap.clone() {
        None => false,
        Some(call) => {
            let chain = &context.chain_storage;
            let maybe_json = chain.read().unwrap().fetch(&call.cap_token).unwrap();
            let grant = match maybe_json {
                Some(content) => CapTokenGrant::try_from(content).unwrap(),
                None => return false,
            };
            grant.verify(call.cap_token.clone(), call.caller, &call.signature)
        }
    }
}

/// InitializationFuture resolves to an Ok(NucleusStatus) or an Err(String).
/// Tracks the nucleus status.
pub struct CallResultFuture {
    context: Arc<Context>,
    zome_call: ZomeFnCall,
}

impl Future for CallResultFuture {
    type Output = Result<JsonString, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();

        if let Some(state) = self.context.state() {
            match state.nucleus().zome_call_result(&self.zome_call) {
                Some(result) => Poll::Ready(result),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::{is_token_the_agent};
    use crate::instance::tests::test_instance_and_context;
    use holochain_core_types::{
        cas::content::Address,
        dna::capabilities::{CapabilityCall}
    };

    #[test]
    fn test_agent_as_token() {
        let dna = test_utils::create_test_dna_with_wat("bad_zome", "test_cap", None);
        let (_, context) = test_instance_and_context(dna, None).expect("Could not initialize test instance");
        let agent_token = Address::from(context.agent_id.key.clone());
        let cap_call = CapabilityCall::new(agent_token, None);
        assert!(is_token_the_agent(context.clone(), &Some(cap_call)));
        let cap_call = CapabilityCall::new(Address::from(""), None);
        assert!(!is_token_the_agent(context, &Some(cap_call)));
    }
}