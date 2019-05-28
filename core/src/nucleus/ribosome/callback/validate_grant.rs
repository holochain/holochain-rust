use crate::{
    context::Context,
    nucleus::{
        ribosome::{
            self,
            callback::{Callback, CallbackParams, CallbackResult},
            runtime::WasmCallData,
            Defn,
        },
        CallbackFnCall,
    },
};
use holochain_core_types::{error::HolochainError, json::JsonString};
use std::sync::Arc;

pub fn validate_grant(
    context: Arc<Context>,
    zome: &str,
    parameters: &CallbackParams,
) -> CallbackResult {
    let params = match parameters {
        CallbackParams::ValidateGrant(params) => params,
        _ => return CallbackResult::NotImplemented("validate_grant/1".into()),
    };

    let call = CallbackFnCall::new(
        zome,
        &Callback::ValidateGrant.as_str().to_string(),
        JsonString::from(params),
    );

    match ribosome::run_dna(
        Some(call.clone().parameters.to_bytes()),
        WasmCallData::new_callback_call(context, call),
    ) {
        Ok(call_result) => CallbackResult::ValidateGrantResult(call_result.into()),
        Err(err) => CallbackResult::Fail(err.to_string()),
    }
}
