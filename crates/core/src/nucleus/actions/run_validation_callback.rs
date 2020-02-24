use crate::{
    context::Context,
    nucleus::{
        CallbackFnCall,
    },
    wasm_engine::{self, runtime::WasmCallData},
    // NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::validation::ValidationResult;
use holochain_core_types::validation::ValidationError;
use holochain_core_types::error::HolochainError;
use holochain_persistence_api::cas::content::Address;
use std::sync::Arc;
use holochain_wasm_types::WasmError;
use std::string::ToString;
use holochain_metrics::with_latency_publishing;

/// Validation callback action creator.
/// Spawns a thread in which a WASM Ribosome runs the custom validation function defined by
/// `zome_call`.
/// Dispatches an `Action::ReturnValidationResult` after completion of the WASM call.
/// Returns a future that waits for the result to appear in the nucleus state.
#[no_autotrace] // TODO: get autotrace working for this future
// TODO: uncommenting this causes line numbers to disappear in this file for compiler errors
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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

            match wasm_engine::run_dna(
                WasmCallData::new_callback_call(cloned_context, call),
                Some(call.clone().parameters.to_bytes()),
            ) {
                Ok(v) => v,
                // TODO: have "not matching schema" be its own error
                Err(HolochainError::Wasm(wasm_error)) => {
                    if wasm_error == WasmError::ArgumentDeserializationFailed {
                        ValidationResult::Err(
                            ValidationError::Fail(
                            "JSON object does not match entry schema".into()
                        )
                        )
                    } else {
                        // an unknown error from the ribosome should panic rather than
                        // silently failing validation
                        panic!(wasm_error)
                    }
                }
                Err(error) => panic!(error.to_string()), // same here
            }
        },
        ()
    )
}
