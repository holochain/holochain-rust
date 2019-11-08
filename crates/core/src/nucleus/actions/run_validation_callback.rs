use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{
        ribosome::{self, runtime::WasmCallData},
        validation::{ValidationError, ValidationResult},
        CallbackFnCall,
    },
};
use futures::{future::Future, task::Poll};
use holochain_core_types::{error::HolochainError, ugly::lax_send_sync};
use holochain_persistence_api::{cas::content::Address, hash::HashString};
use snowflake;
use std::{pin::Pin, sync::Arc};

use holochain_metrics::Metric;

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
    let clone_address = address.clone();
    let cloned_context = context.clone();

    let clock = std::time::SystemTime::now();
    let call2 = call.clone();

    context.clone().spawn_thread(move || {
        let validation_result: ValidationResult = match ribosome::run_dna(
            Some(call2.clone().parameters.to_bytes()),
            WasmCallData::new_callback_call(cloned_context.clone(), call2),
        ) {
            Ok(call_result) => {
                if call_result.is_null() {
                    Ok(())
                } else {
                    Err(ValidationError::Fail(call_result.to_string()))
                }
            }
            // TODO: have "not matching schema" be its own error
            Err(HolochainError::RibosomeFailed(error_string)) => {
                if error_string == "Argument deserialization failed" {
                    Err(ValidationError::Error(
                        String::from("JSON object does not match entry schema").into(),
                    ))
                } else {
                    // an unknown error from the ribosome should panic rather than
                    // silently failing validation
                    panic!(error_string)
                }
            }
            Err(error) => panic!(error.to_string()), // same here
        };

        lax_send_sync(
            cloned_context.action_channel().clone(),
            ActionWrapper::new(Action::ReturnValidationResult((
                (id, clone_address),
                validation_result,
            ))),
            "run_validation_callback",
        );
    });

    let awaited = ValidationCallbackFuture {
        context: context.clone(),
        key: (id, address),
    }
    .await;

    let metric_name = format!("{}.{}.latency", call.zome_name, call.fn_name);
    let latency = clock.elapsed().unwrap().as_millis();
    let metric = Metric::new(metric_name.as_str(), latency as f64);
    context.metric_publisher.write().unwrap().publish(&metric);
    awaited
}

/// ValidationFuture resolves to an Ok(ActionWrapper) or an Err(error_message:String).
/// Tracks the state for ValidationResults.
pub struct ValidationCallbackFuture {
    context: Arc<Context>,
    key: (snowflake::ProcessUniqueId, HashString),
}

impl Future for ValidationCallbackFuture {
    type Output = ValidationResult;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if !self.context.is_action_channel_open() {
            return Poll::Ready(Err(ValidationError::Error(HolochainError::LifecycleError(
                "ValidationCallbackFuture".to_string(),
            ))));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        if let Some(state) = self.context.try_state() {
            match state.nucleus().validation_results.get(&self.key) {
                Some(result) => Poll::Ready(result.clone()),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
