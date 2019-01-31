use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::RECV_DEFAULT_TIMEOUT_MS,
    nucleus::{
        ribosome::{
            api::ZomeApiResult,
            fn_call::{do_call, make_cap_call, ZomeFnCall},
            Runtime,
        },
        state::NucleusState,
    },
};
use holochain_core_types::{cas::content::Address, error::HolochainError, json::JsonString};
use holochain_wasm_utils::api_serialization::{ZomeFnCallArgs, THIS_INSTANCE};
use jsonrpc_lite::JsonRpc;
use snowflake::ProcessUniqueId;
use std::{
    convert::TryFrom,
    sync::{mpsc::channel, Arc},
};
use wasmi::{RuntimeArgs, RuntimeValue};

// ZomeFnCallArgs to ZomeFnCall
impl ZomeFnCall {
    fn from_args(context: Arc<Context>, args: ZomeFnCallArgs) -> Self {
        let cap_call = make_cap_call(
            context.clone(),
            args.cap_token,
            Address::from(context.agent_id.key.clone()),
            &args.fn_name,
            args.fn_args.clone(),
        );
        ZomeFnCall::new(&args.zome_name, cap_call, &args.fn_name, args.fn_args)
    }
}

/// HcApiFuncIndex::CALL function code
/// args: [0] encoded MemoryAllocation as u64
/// expected complex argument: {zome_name: String, cap_token: Address, fn_name: String, args: String}
/// args from API call are converted into a ZomeFnCall
/// Launch an Action::Call with newly formed ZomeFnCall-
/// Waits for a ZomeFnResult
/// Returns an HcApiReturnCode as I64
pub fn invoke_call(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let zome_call_data = runtime.zome_call_data()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let input = match ZomeFnCallArgs::try_from(args_str.clone()) {
        Ok(input) => input,
        // Exit on error
        Err(_) => {
            zome_call_data.context.log(format!(
                "err/zome: invoke_call failed to deserialize: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let result = if input.instance_handle == String::from(THIS_INSTANCE) {
        // ZomeFnCallArgs to ZomeFnCall
        let zome_call = ZomeFnCall::from_args(zome_call_data.context.clone(), input.clone());

        // Don't allow recursive calls
        if zome_call.same_fn_as(&zome_call_data.zome_call) {
            return ribosome_error_code!(RecursiveCallForbidden);
        }
        local_call(runtime, input)
    } else {
        bridge_call(runtime, input)
    };

    runtime.store_result(result)
}

fn local_call(runtime: &mut Runtime, input: ZomeFnCallArgs) -> Result<JsonString, HolochainError> {
    let zome_call_data = runtime.zome_call_data().map_err(|_| {
        HolochainError::ErrorGeneric(
            "expecting zome call data in local call not null call".to_string(),
        )
    })?;
    // ZomeFnCallArgs to ZomeFnCall
    let zome_call = ZomeFnCall::from_args(zome_call_data.context.clone(), input);
    // Create Call Action
    let action_wrapper = ActionWrapper::new(Action::Call(zome_call.clone()));
    // Send Action and block
    let (sender, receiver) = channel();
    crate::instance::dispatch_action_with_observer(
        zome_call_data.context.action_channel(),
        zome_call_data.context.observer_channel(),
        action_wrapper.clone(),
        move |state: &crate::state::State| {
            // Observer waits for a ribosome_call_result
            let maybe_result = state.nucleus().zome_call_result(&zome_call);
            match maybe_result {
                Some(result) => {
                    // @TODO never panic in wasm
                    // @see https://github.com/holochain/holochain-rust/issues/159
                    sender
                        .send(result)
                        // the channel stays connected until the first message has been sent
                        // if this fails that means that it was called after having returned done=true
                        .expect("observer called after done");

                    true
                }
                None => false,
            }
        },
    );
    // TODO #97 - Return error if timeout or something failed
    // return Err(_);

    receiver
        .recv_timeout(RECV_DEFAULT_TIMEOUT_MS)
        .expect("observer dropped before done")
}

fn bridge_call(runtime: &mut Runtime, input: ZomeFnCallArgs) -> Result<JsonString, HolochainError> {
    let zome_call_data = runtime.zome_call_data().map_err(|_| {
        HolochainError::ErrorGeneric(
            "expecting zome call data in bridge call not null call".to_string(),
        )
    })?;
    let container_api =
        zome_call_data
            .context
            .container_api
            .clone()
            .ok_or(HolochainError::ConfigError(
                "No container API in context".to_string(),
            ))?;

    let method = format!(
        "{}/{}/{}",
        input.instance_handle, input.zome_name, input.fn_name
    );

    let handler = container_api.write().unwrap();

    let id = ProcessUniqueId::new();
    let request = format!(
        r#"{{"jsonrpc": "2.0", "method": "{}", "params": {}, "id": "{}"}}"#,
        method, input.fn_args, id
    );

    let response = handler
        .handle_request_sync(&request)
        .ok_or("Bridge call failed".to_string())?;

    let response = JsonRpc::parse(&response)?;

    match response {
        JsonRpc::Success(_) => Ok(JsonString::from(
            serde_json::to_string(&response.get_result().unwrap()).unwrap(),
        )),
        JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(
            serde_json::to_string(&response.get_error().unwrap()).unwrap(),
        )),
        _ => Err(HolochainError::ErrorGeneric(
            "Bridge call failed".to_string(),
        )),
    }
}

/// Reduce Call Action
///   1. Checks for validity of ZomeFnCall
///   2. Execute the exposed Zome function in a separate thread
/// Send the result in a ReturnZomeFunctionResult Action on success or failure like ExecuteZomeFunction
pub(crate) fn reduce_call(
    context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
) {
    // 1.Checks for correctness of ZomeFnCall
    let fn_call = match action_wrapper.action().clone() {
        Action::Call(call) => call,
        _ => unreachable!(),
    };

    if let Some(err) = do_call(context.clone(), state, fn_call.clone()).err() {
        state.zome_calls.insert(fn_call.clone(), Some(Err(err)));
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    extern crate test_utils;
    extern crate wabt;

    use crate::nucleus::ribosome::api::tests::{
        test_function_name, test_parameters, test_zome_name,
    };
    use holochain_core_types::cas::content::Address;
    use holochain_wasm_utils::api_serialization::ZomeFnCallArgs;
    use serde_json;

    /// dummy commit args from standard test entry
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_bad_args_bytes() -> Vec<u8> {
        let args = ZomeFnCallArgs {
            instance_handle: "instance_handle".to_string(),
            zome_name: "zome_name".to_string(),
            cap_token: Address::from("bad cap_token"),
            fn_name: "fn_name".to_string(),
            fn_args: "fn_args".to_string(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_args_bytes() -> Vec<u8> {
        let args = ZomeFnCallArgs {
            instance_handle: THIS_INSTANCE.to_string(),
            zome_name: test_zome_name(),
            cap_token: Address::from("test_token"),
            fn_name: test_function_name(),
            fn_args: test_parameters(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

}
