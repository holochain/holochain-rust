use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{
        ribosome::{self, runtime::WasmCallData},
        validation::{ValidationError, ValidationResult},
        CallbackFnCall,
    },
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{cas::content::Address, error::HolochainError, hash::HashString};
use snowflake;
use std::{pin::Pin, sync::Arc, thread};

/// Validation callback action creator.
/// Spawns a thread in which a WASM Ribosome runs the custom validation function defined by
/// `zome_call`.
/// Dispatches an `Action::ReturnValidationResult` after completion of the WASM call.
/// Returns a future that waits for the result to appear in the nucleus state.
pub async fn run_validation_callback(
    address: Address,
    call: CallbackFnCall,
    context: &Arc<Context>,
) -> ValidationResult {
    let id = snowflake::ProcessUniqueId::new();
    let dna_name = context
        .state()
        .unwrap()
        .nucleus()
        .dna
        .as_ref()
        .unwrap()
        .name
        .clone();
    let wasm = context
        .get_wasm(&call.zome_name)
        .ok_or(ValidationError::NotImplemented)?;

    let clone_address = address.clone();
    let cloned_context = context.clone();
    thread::spawn(move || {
        let validation_result: ValidationResult = match ribosome::run_dna(
            wasm.code.clone(),
            Some(call.clone().parameters.into_bytes()),
            WasmCallData::new_callback_call(cloned_context.clone(), dna_name, call),
        ) {
            Ok(call_result) => match call_result.is_null() {
                true => Ok(()),
                false => Err(ValidationError::Fail(call_result.to_string())),
            },
            // TODO: have "not matching schema" be its own error
            Err(HolochainError::RibosomeFailed(error_string)) => {
                if error_string == "Argument deserialization failed" {
                    Err(ValidationError::Error(String::from(
                        "JSON object does not match entry schema",
                    )))
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
