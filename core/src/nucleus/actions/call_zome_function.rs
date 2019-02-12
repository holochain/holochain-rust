extern crate futures;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{
        is_fn_public,
        ribosome::{self, api::call::check_capability},
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
use std::thread;

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
