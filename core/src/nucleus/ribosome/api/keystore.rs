use crate::{
    context::Context,
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
};
use holochain_core_types::{
    error::{HcResult, HolochainError},
    json::JsonString,
};
use holochain_wasm_utils::api_serialization::keystore::KeystoreListResult;
use jsonrpc_lite::JsonRpc;
use serde_json::{self, Value};
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
        .ok_or(HolochainError::new("Callback failed"))?;

    let response = JsonRpc::parse(&response)?;

    match response {
        JsonRpc::Success(_) => Ok(JsonString::from(
            serde_json::to_string(&response.get_result().unwrap()).unwrap(),
        )),
        JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(
            serde_json::to_string(&response.get_error().unwrap()).unwrap(),
        )),
        _ => Err(HolochainError::ErrorGeneric(format!(
            "{} call failed",
            method
        ))),
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

    runtime.store_result(Ok(KeystoreListResult { ids: string_list }))
}

pub fn invoke_keystore_new_random(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;

    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let result = conductor_callback(
        "agent/keystore/add_random_seed",
        &args_str.to_string(),
        context.clone(),
    );
    match result {
        Ok(_) => (),
        Err(err) => {
            context.log(format!(
                "err/zome: agent/keystore/add_random_seed callback failed: {:?}",
                err
            ));
            return ribosome_error_code!(CallbackFailed);
        }
    };
    runtime.store_result(Ok(()))
}

pub fn invoke_keystore_derive_seed(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let result = conductor_callback(
        "agent/keystore/add_seed_from_seed",
        &args_str.to_string(),
        context.clone(),
    );
    match result {
        Ok(_) => (),
        Err(err) => {
            context.log(format!(
                "err/zome: agent/keystore/add_seed_from_seed callback failed: {:?}",
                err
            ));
            return ribosome_error_code!(CallbackFailed);
        }
    };

    runtime.store_result(Ok(()))
}

pub fn invoke_keystore_derive_key(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let result = conductor_callback(
        "agent/keystore/add_key_from_seed",
        &args_str.to_string(),
        context.clone(),
    );
    let string: String = match result {
        Ok(json_string) => {
            context.log(format!(
                "debug/zome: keystore_add_key_from_seed json_string:{:?}",
                json_string
            ));
            let value: Value = serde_json::from_str(&json_string.to_string()).unwrap();
            value["pub_key"].to_string()
        }
        Err(err) => {
            context.log(format!(
                "err/zome: agent/keystore/add_key_from_seed callback failed: {:?}",
                err
            ));
            return ribosome_error_code!(CallbackFailed);
        }
    };

    context.log(format!(
        "debug/zome: pubkey derive of args:{:?} is:{:?}",
        args_str, string
    ));
    runtime.store_result(Ok(string))
}

pub fn invoke_keystore_sign(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let result = conductor_callback(
        "agent/keystore/sign",
        &args_str.to_string(),
        context.clone(),
    );
    let string: String = match result {
        Ok(json_string) => {
            context.log(format!(
                "debug/zome: keystore_sign json_string:{:?}",
                json_string
            ));

            let value: Value = serde_json::from_str(&json_string.to_string()).unwrap();
            value["signature"].as_str().unwrap().to_owned()
        }
        Err(err) => {
            context.log(format!(
                "err/zome: agent/keystore/sign callback failed: {:?}",
                err
            ));
            return ribosome_error_code!(CallbackFailed);
        }
    };

    context.log(format!(
        "debug/zome: signature of args:{:?} is:{:?}",
        args_str, string
    ));

    runtime.store_result(Ok(string))
}
