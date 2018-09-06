use action::{Action, ActionWrapper};
use context::Context;
use error::HolochainError;
use holochain_dna::zome::capabilities::Membrane;
use instance::{Observer, RECV_DEFAULT_TIMEOUT_MS};
use nucleus::{
    launch_zome_fn_call,
    ribosome::api::{runtime_allocate_encode_str, runtime_args_to_utf8, HcApiReturnCode, Runtime},
    state::NucleusState,
    FunctionCall,
};
use serde_json;
use std::sync::{
    mpsc::{channel, Sender},
    Arc,
};
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// Struct for input data received when Call API function is invoked
#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub struct ZomeCallArgs {
    pub zome_name: String,
    pub cap_name: String,
    pub fn_name: String,
    pub fn_args: String,
}

// ZomeCallArgs to FunctionCall
impl FunctionCall {
    fn from_args(args: ZomeCallArgs) -> Self {
        FunctionCall::new(
            &args.zome_name,
            &args.cap_name,
            &args.fn_name,
            &args.fn_args,
        )
    }
}

/// Plan:
/// args from API converts into a FunctionCall
/// Invoke launch a Action::Call with said FunctionCall
/// Waits for a Action::ReturnZomeFunctionResult since action will launch a ExecuteZomeFunction
/// on success.
///
/// Action::Call reducer does:
///   Checks for correctness of FunctionCall
///   Checks for correct access to Capability
///   Launch a ExecuteZomeFunction with FunctionCall
///
///

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

    // ZomeCallArgs to FunctionCall
    let fn_call = FunctionCall::from_args(input);

    // Create Call Action
    let action_wrapper = ActionWrapper::new(Action::Call(fn_call.clone()));
    println!(" !! Looking for: {:?}", action_wrapper);
    // Send Action and block
    let (sender, receiver) = channel();
    ::instance::dispatch_action_with_observer(
        &runtime.action_channel,
        &runtime.observer_channel,
        action_wrapper.clone(),
        move |state: &::state::State| {
            // Observer waits for a ribosome_call_result
            let opt_res = state.nucleus().ribosome_call_result(&fn_call);
            println!("\t opt_res: {:?}", opt_res);
            match opt_res {
                Some(res) => {
                    // @TODO never panic in wasm
                    // @see https://github.com/holochain/holochain-rust/issues/159
                    sender
                        .send(res)
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
    let action_result = receiver
        .recv_timeout(RECV_DEFAULT_TIMEOUT_MS)
        .expect("observer dropped before done");
    println!("invoke_call: Done: {:?}", action_result);

    // action_result is Action::ReturnZomeFunctionResult(result))

    match action_result {
        Ok(res) => {
            // let res = runtime.state().nucleus().ribosome_call_result(fn_call);
            // serialize, allocate and encode result
            //            let json = res.to_json();
            //            match json {
            //                Ok(j) => runtime_allocate_encode_str(runtime, &j),
            //                Err(_) => Ok(Some(RuntimeValue::I32(HcApiReturnCode::ErrorJson as i32))),
            //            }
            runtime_allocate_encode_str(runtime, &res)
        }
        Err(_) => Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ErrorActionResult as i32,
        ))),
    }
}

/// Reduce Call Action
/// Execute an exposed Zome function in a separate thread and send the result in
/// a ReturnZomeFunctionResult Action on success or failure
pub(crate) fn reduce_call(
    context: Arc<Context>,
    state: &mut NucleusState,
    action_wrapper: &ActionWrapper,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) {
    let fn_call = match action_wrapper.action().clone() {
        Action::Call(call) => call,
        _ => unreachable!(),
    };

    // println!("fn_call = {:?}", fn_call);

    // Get Capability
    let maybe_cap = state.get_capability(fn_call.clone());
    if let Err(fn_res) = maybe_cap {
        // Send Failed Result
        // println!("fn_res = {:?}", fn_res);
        state
            .ribosome_calls
            .insert(fn_call.clone(), Some(fn_res.result()));
        action_channel
            .send(action_wrapper.clone())
            .expect("action channel to be open in reducer");
        return;
    }
    let cap = maybe_cap.unwrap();

    // println!("cap = {:?}", cap);

    // Check if we have permission to call that Zome function
    // FIXME is this enough?
    if cap.cap_type.membrane != Membrane::Zome {
        // Send Failed Result
        state.ribosome_calls.insert(
            fn_call.clone(),
            Some(Err(HolochainError::DoesNotHaveCapabilityToken)),
        );
        // println!("fn_res = {:?}", fn_res);
        action_channel
            .send(action_wrapper.clone())
            // .send(ActionWrapper::new(Action::ReturnZomeFunctionResult(fn_res)))
            .expect("action channel to be open in reducer");
        return;
    }

    // println!("cap = {:?}", cap);

    // Prepare call
    state.ribosome_calls.insert(fn_call.clone(), None);

    // Launch thread with function call
    launch_zome_fn_call(
        context,
        fn_call,
        action_channel,
        observer_channel,
        &cap.code,
        state.dna.clone().unwrap().name,
    );
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use super::*;
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function_runtime, ZomeAPIFunction},
        Defn,
    };
    use serde_json;

    /// dummy commit args from standard test entry
    pub fn test_bad_args_bytes() -> Vec<u8> {
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
        let (runtime, _) =
            test_zome_api_function_runtime(ZomeAPIFunction::Call.as_str(), test_args_bytes());
        println!("test_call_round_trip");

        assert_eq!(runtime.result, format!(r#""#),);
    }

    #[test]
    fn test_call_no_zome() {
        let (runtime, _) =
            test_zome_api_function_runtime(ZomeAPIFunction::Call.as_str(), test_bad_args_bytes());
        println!("test_call_round_trip");

        assert_eq!(runtime.result, format!(r#""#),);
    }

}
