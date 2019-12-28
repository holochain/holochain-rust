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

use holochain_metrics::with_latency_publishing;

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
    let metric_name_prefix = format!(
        "run_validation_callback.{}.{}",
        call.zome_name, call.fn_name
    );

    with_latency_publishing!(
        metric_name_prefix,
        context.metric_publisher,
        |()| {
            let cloned_context = context.clone();

            match ribosome::run_dna(
                Some(call.clone().parameters.to_bytes()),
                WasmCallData::new_callback_call(cloned_context, call),
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
            }
        },
        ()
    )
}
