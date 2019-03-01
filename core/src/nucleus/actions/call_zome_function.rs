use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{
        ribosome::{self, WasmCallData},
        ZomeFnCall, ZomeFnResult,
    },
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::error::HolochainError;
use std::{pin::Pin, sync::Arc};

use holochain_core_types::{
    dna::capabilities::CapabilityCall, entry::cap_entries::CapTokenGrant, json::JsonString,
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

/// Execution of zome calls
/// This function is kicking off the execution of a given zome function with given parameters.
/// It dispatches two actions:
/// * `SignalZomeFunctionCall`: after passing checks and before actually starting the Ribosome,
/// * `ReturnZomeFunctionResult`: asynchronously after execution of the Ribosome has completed.
///
/// It is doing pre-checks (such as the capability check) synchronously but then spawns a new
/// thread to run the Ribosome in.
///
/// Being an async function, it returns a future that is polling the instance's State until
/// the call result gets added there through the `RetunrZomeFunctionResult` action.
///
/// Use Context::block_on to wait for the call result.
pub async fn call_zome_function(
    zome_call: ZomeFnCall,
    context: &Arc<Context>,
) -> Result<JsonString, HolochainError> {
    // Get DNA name and WASM code from state.
    // This happens in a code block to scope the read-lock that we acquire from the state
    // so that it drops the lock and frees the state for mutation.
    // If we would leak (and move) the lock into the Ribosome thread below, it would lead to a
    // dead-lock since the existence of this read-lock prevents the redux loop from writing to
    // the state..
    let (dna_name, code) = {
        let state = context.state().ok_or(HolochainError::ErrorGeneric(
            "Context not initialized".to_string(),
        ))?;
        let nucleus_state = state.nucleus();
        let dna = nucleus_state
            .dna
            .as_ref()
            .ok_or(HolochainError::DnaMissing)?;

        // Validate the call
        // 1. make sure the zome and function exists
        let _ = dna
            .get_function_with_zome_name(&zome_call.zome_name, &zome_call.fn_name)
            .map_err(HolochainError::Dna)?;

        let zome = dna
            .get_zome(&zome_call.zome_name)
            .map_err(HolochainError::Dna)?;

        // 2. make sure caller is allowed to call the function
        let public = zome.is_fn_public(&zome_call.fn_name);

        if !public && !check_capability(context.clone(), &zome_call.clone()) {
            return Err(HolochainError::CapabilityCheckFailed);
        }

        // 3. read data needed to execute the zome function
        let dna_name = dna.name.clone();
        let code = dna
            .get_wasm_from_zome_name(zome_call.zome_name.clone())
            .expect("zome not found, Should have failed before when getting capability.")
            .code
            .clone();

        (dna_name, code)
    };

    // Clone context and call data for the Ribosome thread
    let context_clone = context.clone();
    let zome_call_clone = zome_call.clone();

    // Signal (currently mainly to the nodejs_waiter) that we are about to start a zome function:
    context
        .action_channel()
        .send(ActionWrapper::new(Action::SignalZomeFunctionCall(
            zome_call.clone(),
        )))
        .expect("action channel to be open");

    let _ = thread::spawn(move || {
        // Have Ribosome spin up DNA and call the zome function
        let call_result = ribosome::run_dna(
            code,
            Some(zome_call_clone.clone().parameters.into_bytes()),
            WasmCallData::new_zome_call(
                context_clone.clone(),
                dna_name.clone(),
                zome_call_clone.clone(),
            ),
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
        Some(call) => context.agent_id.pub_sign_key == call.cap_token.to_string(),
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

/// CallResultFuture resolves to an Result<JsonString, HolochainError>.
/// Tracks the nucleus State, waiting for a result to the given zome function call to appear.
pub struct CallResultFuture {
    context: Arc<Context>,
    zome_call: ZomeFnCall,
}

impl Future for CallResultFuture {
    type Output = Result<JsonString, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        // With our own executor implementation in Context::block_on we actually
        // wouldn't need the waker since this executor is attached to the redux loop
        // and re-polls after every State mutation.
        // Leaving this in to be safe against running this future in another executor.
        lw.wake();

        if let Some(state) = self.context.state() {
            match state.nucleus().zome_call_result(&self.zome_call) {
                Some(result) => Poll::Ready(result),
                None => Poll::Pending,
            }
        } else {
            Poll::Ready(Err(HolochainError::ErrorGeneric(
                "State not initialized".to_string(),
            )))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::is_token_the_agent;
    use crate::instance::tests::test_instance_and_context;
    use holochain_core_types::{cas::content::Address, dna::capabilities::CapabilityCall};

    #[test]
    fn test_agent_as_token() {
        let dna = test_utils::create_test_dna_with_wat("bad_zome", "test_cap", None);
        let (_, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");
        let agent_token = Address::from(context.agent_id.pub_sign_key.clone());
        let cap_call = CapabilityCall::new(agent_token, None);
        assert!(is_token_the_agent(context.clone(), &Some(cap_call)));
        let cap_call = CapabilityCall::new(Address::from(""), None);
        assert!(!is_token_the_agent(context, &Some(cap_call)));
    }
}
