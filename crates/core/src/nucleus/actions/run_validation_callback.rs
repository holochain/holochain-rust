use crate::{
    context::Context,
    nucleus::{
        ribosome::{self, runtime::WasmCallData},
        validation::{ValidationError, ValidationResult},
        CallbackFnCall,
    },
};
use holochain_core_types::error::HolochainError;
use holochain_persistence_api::cas::content::Address;
use std::sync::Arc;

use holochain_metrics::Metric;

/// Validation callback action creator.
/// Spawns a thread in which a WASM Ribosome runs the custom validation function defined by
/// `zome_call`.
/// Dispatches an `Action::ReturnValidationResult` after completion of the WASM call.
/// Returns a future that waits for the result to appear in the nucleus state.
pub async fn run_validation_callback(
    _address: Address,
    call: CallbackFnCall,
    context: &Arc<Context>,
) -> ValidationResult {
    let cloned_context = context.clone();

    let clock = std::time::SystemTime::now();
    let call2 = call.clone();

    let validation_result: ValidationResult = match ribosome::run_dna(
        Some(call2.clone().parameters.to_bytes()),
        WasmCallData::new_callback_call(cloned_context, call2),
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

    let metric_name = format!("{}.{}.latency", call.zome_name, call.fn_name);
    let latency = clock.elapsed().unwrap().as_millis();
    let metric = Metric::new(metric_name.as_str(), latency as f64);
    context.metric_publisher.write().unwrap().publish(&metric);

    validation_result
}
