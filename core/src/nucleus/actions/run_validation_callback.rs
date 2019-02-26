use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{
        ribosome::{self, runtime::WasmCallData},
        ZomeFnCall,
    },
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{
    cas::content::Address,
    error::HolochainError,
    hash::HashString,
};
use snowflake;
use std::{pin::Pin, sync::Arc, thread};
use crate::nucleus::state::ValidationResult;
use crate::nucleus::state::ValidationError;

/// ValidateEntry Action Creator
/// This is the high-level validate function that wraps the whole validation process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// 1. Checks if the entry type is either an app entry type defined in the DNA or a system entry
///    type that should be validated. Bails early if not.
/// 2. Checks if the entry's address matches the address in given header provided by
///    the validation package.
/// 3. Validates provenances given in the header by verifying the cryptographic signatures
///    against the source agent addresses.
/// 4. Finally spawns a thread to run the custom validation callback in a Ribosome.
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub async fn run_validation_callback(
    address: Address,
    zome_call: ZomeFnCall,
    context: &Arc<Context>,
) -> ValidationResult {
    let id = snowflake::ProcessUniqueId::new();
    let dna_name = context.state().unwrap().nucleus().dna.as_ref().unwrap().name.clone();
    let wasm = context.get_wasm(&zome_call.zome_name)
        .ok_or(ValidationError::NotImplemented)?;


    let clone_address = address.clone();
    let cloned_context = context.clone();
    thread::spawn(move || {
        let validation_result : ValidationResult = match ribosome::run_dna(
            wasm.code.clone(),
            Some(zome_call.clone().parameters.into_bytes()),
            WasmCallData::new_zome_call(cloned_context.clone(), dna_name, zome_call),
        ) {
            Ok(call_result) => match call_result.is_null() {
                true => Ok(()),
                false => Err(ValidationError::Fail(call_result.to_string())),
            },
            // TODO: have "not matching schema" be its own error
            Err(HolochainError::RibosomeFailed(error_string)) => {
                if error_string == "Argument deserialization failed" {
                    Err(ValidationError::Error(String::from("JSON object does not match entry schema")))
                } else {
                    Err(ValidationError::Error(error_string))
                }
            }
            Err(error) => Err(ValidationError::Error(error.to_string())),
        };


        cloned_context
            .action_channel()
            .send(ActionWrapper::new(Action::ReturnValidationResult((
                (id, clone_address),
                validation_result,
            ))))
            .expect("action channel to be open in reducer");
    });

    await!(ValidationCallbackFuture {
        context: context.clone(),
        key: (id, address),
    })
}

/// ValidationFuture resolves to an Ok(ActionWrapper) or an Err(error_message:String).
/// Tracks the state for ValidationResults.
pub struct ValidationCallbackFuture {
    context: Arc<Context>,
    key: (snowflake::ProcessUniqueId, HashString),
}

impl Future for ValidationCallbackFuture {
    type Output = ValidationResult;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        if let Some(state) = self.context.state() {
            match state.nucleus().validation_results.get(&self.key) {
                Some(result) => Poll::Ready(result.clone()),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
