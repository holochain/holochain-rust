use action::{Action, ActionWrapper};
use agent::state::ActionResponse;
use json::ToJson;
use nucleus::FunctionCall;
use nucleus::ribosome::{
    api::{runtime_allocate_encode_str, runtime_args_to_utf8, HcApiReturnCode, Runtime},
    callback::{validate_commit::validate_commit, CallbackParams, CallbackResult},
};
use instance::RECV_DEFAULT_TIMEOUT_MS;
use serde_json;
use std::sync::mpsc::channel;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// Struct for input data received when Call API function is invoked
#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub struct ZomeCallArgs {
    pub zome_name: String,
    pub cap_name: String,
    pub fn_name: String,
    pub fn_args: String,
}

/// HcApiFuncIndex::CALL function code
/// args: [0] encoded MemoryAllocation as u32
/// expected complex argument: r#"{"entry_type_name":"post","entry_content":"hello"}"#
/// Returns an HcApiReturnCode as I32
pub fn invoke_call(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime_args_to_utf8(&runtime, &args);
    let input: ZomeCallArgs = match serde_json::from_str(&args_str) {
        Ok(input) => input,
        // Exit on error
        Err(_) => {
            // Return Error code in i32 format
            return Ok(Some(RuntimeValue::I32(HcApiReturnCode::ErrorJson as i32)));
        }
    };

    // Create Call Action
    let action_wrapper = ActionWrapper::new(Action::Call(input));
    println!(" !! Looking for: {:?}", action_wrapper);
    // Send Action and block for result
    let (sender, receiver) = channel();
    ::instance::dispatch_action_with_observer(
        &runtime.action_channel,
        &runtime.observer_channel,
        action_wrapper.clone(),
        move |state: &::state::State| {
            let mut actions_copy = state.agent().actions();
            println!("actions_copy: {:?}", actions_copy);
            match actions_copy.remove(&action_wrapper) {
                Some(v) => {
                    // @TODO never panic in wasm
                    // @see https://github.com/holochain/holochain-rust/issues/159
                    sender
                        .send(v)
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

    println!("invoke_call: waiting...");
    let action_result = receiver.recv_timeout(RECV_DEFAULT_TIMEOUT_MS).expect("observer dropped before done");
    println!("invoke_call: Done: {:?}", action_result);

    match action_result {
        ActionResponse::Call(_) => {
            // serialize, allocate and encode result
            let json = action_result.to_json();
            match json {
                Ok(j) => runtime_allocate_encode_str(runtime, &j),
                Err(_) => Ok(Some(RuntimeValue::I32(HcApiReturnCode::ErrorJson as i32))),
            }
        }
        _ => Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ErrorActionResult as i32,
        ))),
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use super::*;
    use hash_table::entry::tests::test_entry;
    use key::Key;
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function_runtime, ZomeAPIFunction},
        Defn,
    };
    use serde_json;

    /// dummy commit args from standard test entry
    pub fn test_bad_args_bytes() -> Vec<u8> {
        let e = test_entry();
        let args = ZomeCallArgs {
            zome_name: "zome_name".to_string(),
            cap_name: "cap_name".to_string(),
            fn_name: "fn_name".to_string(),
            fn_args: "fn_args".to_string(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    pub fn test_args_bytes() -> Vec<u8> {
        let e = test_entry();
        let args = ZomeCallArgs {
            zome_name: "test_zome".to_string(),
            cap_name: "".to_string(),
            fn_name: "test".to_string(),
            fn_args: "".to_string(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }


    /// test that we can round trip bytes through a commit action and get the result from WASM
    #[test]
    fn test_call_round_trip() {
        let (runtime, _) = test_zome_api_function_runtime(
            ZomeAPIFunction::Call.as_str(),
            test_args_bytes(),
        );
        println!("test_call_round_trip");

        assert_eq!(
            runtime.result,
            format!(r#"{{"hash":"{}"}}"#, test_entry().key()) + "\u{0}",
        );
    }

}
