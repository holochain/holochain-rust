use crate::{context::Context};
use holochain_core_types::error::{HcResult, HolochainError};

use holochain_json_api::json::JsonString;

use holochain_wasm_types::{
    keystore::KeystoreListResult, wasm_string::WasmString,
};
use holochain_wasmer_host::*;
use jsonrpc_lite::JsonRpc;
use serde_json::{self, Value};
use snowflake::ProcessUniqueId;
use std::sync::Arc;
use crate::workflows::WorkflowResult;

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn conductor_callback<S: Into<String>>(
    method: S,
    params: S,
    context: Arc<Context>,
) -> HcResult<JsonString> {
    let conductor_api = context.conductor_api.clone();

    let handler = conductor_api.get().write().unwrap();

    let method = method.into();
    let id = ProcessUniqueId::new();
    let request = format!(
        r#"{{"jsonrpc": "2.0", "method": "{}", "params": {}, "id": "{}"}}"#,
        method,
        params.into(),
        id
    );

    let response = handler
        .handle_request_sync(&request)
        .ok_or("Callback failed")?;

    let response = JsonRpc::parse(&response)?;

    match response {
        JsonRpc::Success(_) => Ok(JsonString::from(response.get_result().unwrap().to_owned())),
        JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(
            serde_json::to_string(&response.get_error().unwrap()).unwrap(),
        )),
        _ => Err(HolochainError::ErrorGeneric(format!(
            "{} call failed",
            method
        ))),
    }
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn keystore_list_workflow(context: Arc<Context>, _: &()) -> WorkflowResult<KeystoreListResult> {
    let result = conductor_callback("agent/keystore/list", "{}", Arc::clone(&context));
    let string_list: Vec<String> = match result {
        Ok(json_array) => serde_json::from_str(&json_array.to_string()).unwrap(),
        Err(err) => {
            log_error!(
                context,
                "zome: agent/keystore/list callback failed: {:?}",
                err
            );
            return Err(WasmError::CallbackFailed)?;
        }
    };

    Ok(KeystoreListResult { ids: string_list })
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn keystore_new_random_workflow(context: Arc<Context>, args_str: &WasmString) -> WorkflowResult<()> {
    let result = conductor_callback(
        "agent/keystore/add_random_seed",
        &args_str.to_string(),
        Arc::clone(&context),
    );
    match result {
        Ok(_) => (),
        Err(err) => {
            log_error!(
                context,
                "zome: agent/keystore/add_random_seed callback failed: {:?}",
                err
            );
            return Err(WasmError::CallbackFailed)?;
        }
    };
    Ok(())
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn keystore_derive_seed_workflow(context: Arc<Context>, args_str: &WasmString) -> WorkflowResult<()> {
    let result = conductor_callback(
        "agent/keystore/add_seed_from_seed",
        &args_str.to_string(),
        Arc::clone(&context),
    );
    match result {
        Ok(_) => (),
        Err(err) => {
            log_error!(
                context,
                "zome: agent/keystore/add_seed_from_seed callback failed: {:?}",
                err
            );
            return Err(WasmError::CallbackFailed)?;
        }
    };

    Ok(())
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn keystore_derive_key_workflow(context: Arc<Context>, args_str: &WasmString) -> WorkflowResult<JsonString> {
    let result = conductor_callback(
        "agent/keystore/add_key_from_seed",
        &args_str.to_string(),
        Arc::clone(&context),
    );
    let string: String = match result {
        Ok(json_string) => {
            log_debug!(
                context,
                "zome: keystore_add_key_from_seed json_string:{:?}",
                json_string
            );
            let value: Value = serde_json::from_str(&json_string.to_string()).unwrap();
            value["pub_key"].to_string()
        }
        Err(err) => {
            log_error!(
                context,
                "zome: agent/keystore/add_key_from_seed callback failed: {:?}",
                err
            );
            return Err(WasmError::CallbackFailed)?;
        }
    };

    log_debug!(
        context,
        "zome: pubkey derive of args:{:?} is:{:?}",
        args_str,
        string
    );
    Ok(JsonString::from_json(&string))
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn keystore_sign_workflow(context: Arc<Context>, args_str: &WasmString) -> WorkflowResult<JsonString> {
    let result = conductor_callback("agent/keystore/sign", &args_str.to_string(), Arc::clone(&context));
    let string: String = match result {
        Ok(json_string) => {
            log_debug!(context, "zome: keystore_sign json_string:{:?}", json_string);

            let value: Value = serde_json::from_str(&json_string.to_string()).unwrap();
            value["signature"].as_str().unwrap().to_owned()
        }
        Err(err) => {
            log_error!(
                context,
                "zome: agent/keystore/sign callback failed: {:?}",
                err
            );
            return Err(WasmError::CallbackFailed)?;
        }
    };

    log_debug!(
        context,
        "zome: signature of args:{:?} is:{:?}",
        args_str,
        string
    );

    Ok(JsonString::from_json(&string))
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn keystore_get_public_key_workflow(
    context: Arc<Context>,
    args_str: &WasmString,
) -> Result<JsonString, HolochainError> {
    let result = conductor_callback(
        "agent/keystore/get_public_key",
        &args_str.to_string(),
        Arc::clone(&context),
    );
    let string: String = match result {
        Ok(json_string) => {
            log_debug!(
                context,
                "zome: keystore_get_public_key json_string:{:?}",
                json_string
            );
            let value: Value = serde_json::from_str(&json_string.to_string()).unwrap();
            value["pub_key"].to_string()
        }
        Err(err) => {
            log_error!(
                context,
                "zome: agent/keystore/get_public_key callback failed: {:?}",
                err
            );
            return Err(WasmError::CallbackFailed)?;
        }
    };

    log_debug!(
        context,
        "zome: pubkey for args:{:?} is:{:?}",
        args_str,
        string
    );
    Ok(JsonString::from_json(&string))
}
