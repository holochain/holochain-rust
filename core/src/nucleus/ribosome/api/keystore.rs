use crate::{
    context::Context,
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
};
use holochain_core_types::{
    error::error::{HcResult, HolochainError},
    json::JsonString,
};
use holochain_wasm_utils::api_serialization::keystore::KeystoreListResult;
use jsonrpc_lite::JsonRpc;
use snowflake::ProcessUniqueId;
use std::sync::Arc;
use wasmi::{RuntimeArgs, RuntimeValue};

fn conductor_callback<S: Into<String>>(
    method: S,
    params: S,
    context: Arc<Context>,
) -> HcResult<JsonString> {
    let conductor_api = context.conductor_api.clone();

    let handler = conductor_api.write().unwrap();

    let id = ProcessUniqueId::new();
    let request = format!(
        r#"{{"jsonrpc": "2.0", "method": "{}", "params": {}, "id": "{}"}}"#,
        method.into(),
        params.into(),
        id
    );

    let response = handler
        .handle_request_sync(&request)
        .ok_or(HolochainError::new("Callback failed"))?;

    let response = JsonRpc::parse(&response)?;

    match response {
        JsonRpc::Success(_) => Ok(JsonString::from(
            serde_json::to_string(&response.get_result().unwrap()).unwrap(),
        )),
        JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(
            serde_json::to_string(&response.get_error().unwrap()).unwrap(),
        )),
        _ => Err(HolochainError::ErrorGeneric(
            "sign_one_time call failed".to_string(),
        )),
    }
}

pub fn invoke_keystore_list(runtime: &mut Runtime, _args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    let result = conductor_callback("agent/keystore/list", "{}", context.clone());
    let string_list: Vec<String> = match result {
        Ok(json_array) => serde_json::from_str(&json_array.to_string()).unwrap(),
        Err(err) => {
            context.log(format!(
                "err/zome: agent/keystore/list callback failed: {:?}",
                err
            ));
            return ribosome_error_code!(CallbackFailed);
        }
    };

    runtime.store_result(Ok(KeystoreListResult::new(string_list)))
}

/*
    let conductor_api = context.conductor_api.clone();

    let method = "agent/keystore/list".to_string();
    let params = "{}".to_string();
    let handler = conductor_api.write().unwrap();

    let id = ProcessUniqueId::new();
    let request = format!(
        r#"{{"jsonrpc": "2.0", "method": "{}", "params": {}, "id": "{}"}}"#,
        method, params, id
    );

    let response = match handler.handle_request_sync(&request) {
        Some(response) => response,
        None => {
            context.log(format!("err/zome: agent/keystore/sign_one_time call failed"));
            return ribosome_error_code!(CallbackFailed);
        }
    };

    let response = match JsonRpc::parse(&response) {
        Ok(response) => response,
        Err(err) => {
            context.log(format!("err/zome: agent/keystore/sign_one_time could not parse callback response: {:?}", err));
            return ribosome_error_code!(CallbackFailed);
        }
    };

    let result = match response {
        JsonRpc::Success(_) => Ok(JsonString::from(
            serde_json::to_string(&response.get_result().unwrap()).unwrap(),
        )),
        JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(
            serde_json::to_string(&response.get_error().unwrap()).unwrap(),
        )),
        _ => Err(HolochainError::ErrorGeneric(
            "sign_one_time call failed".to_string(),
        )),
    };
*/
