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
/// expected complex argument: {zome_name: String, cap_name: String, fn_name: String, args: String}
/// Returns an HcApiReturnCode as I32
pub fn invoke_call(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    println!("RuntimeArgs: {:?}", args);
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

    // Don't allow recursive calls
    if fn_call.same_as(&runtime.function_call) {
        return Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ErrorRecursiveCall as i32,
        )));
    }

    // Create Call Action
    let action_wrapper = ActionWrapper::new(Action::Call(fn_call.clone()));
    println!(" !! Looking for: {:?}", fn_call);
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
        return;
    }
    let cap = maybe_cap.unwrap();

    // println!("cap = {:?}", cap);

    // Check if we have permission to call that Zome function
    // TODO #301 - Do real Capability token check
    if cap.cap_type.membrane != Membrane::Zome {
        // Send Failed Result
        state.ribosome_calls.insert(
            fn_call.clone(),
            Some(Err(HolochainError::DoesNotHaveCapabilityToken)),
        );
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

    use holochain_agent::Agent;
    use holochain_dna::Dna;
    use context::Context;
    use persister::SimplePersister;
    use super::*;
    use nucleus::ribosome::{
        api::{
            ZomeAPIFunction,
            tests::{test_zome_api_function_runtime,
                    test_zome_name,
                    test_capability,
                    test_function_name,
                    test_parameters,
                    test_zome_api_function_wasm,
                    test_zome_api_function_call,
            },
        },
        Defn,
    };
    use instance::{
        tests::{test_context_and_logger, test_instance, TestLogger},
        Instance,
    };
    use test_utils::{
        create_test_cap,
        create_test_dna_with_cap,
        test_context,

    };
    use serde_json;
    use std::{
        sync::{Arc, Mutex},
    };

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
            zome_name: test_zome_name(),
            cap_name: test_capability(),
            fn_name: test_function_name(),
            fn_args: test_parameters(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }


    fn create_context() -> Arc<Context> {
         Arc::new(Context {
                    agent: Agent::from_string("alex".to_string()),
                    logger:  Arc::new(Mutex::new(TestLogger { log: Vec::new() })),
                    persister: Arc::new(Mutex::new(SimplePersister::new())),
                })
    }

    fn test_reduce_call(dna: Dna, expected: Result<String, HolochainError>) {
        let context = create_context();

        let zome_call = FunctionCall::new("test_zome", "test_cap", "main", "");
        let zome_call_action = ActionWrapper::new(Action::Call(zome_call.clone()));

        // Set up instance and process the action
        // let instance = Instance::new();
        let instance = test_instance(dna);
        let (sender, receiver) = channel();
        let closure = move |state: &::state::State| {
            // Observer waits for a ribosome_call_result
            let opt_res = state.nucleus().ribosome_call_result(&zome_call);
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
        };

        let observer = Observer {
            sensor: Box::new(closure),
        };

        let mut state_observers: Vec<Observer> = Vec::new();
        state_observers.push(observer);
        let (_, rx_observer) = channel::<Observer>();
        instance.process_action(zome_call_action, state_observers, &rx_observer, &context);

        println!("waiting...");
        let action_result = receiver
            .recv_timeout(RECV_DEFAULT_TIMEOUT_MS)
            .expect("observer dropped before done");
        println!("Done: {:?}", action_result);

        assert_eq!(expected, action_result);
    }

    #[test]
    fn test_call_no_token() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let expected = Err(HolochainError::DoesNotHaveCapabilityToken);
        test_reduce_call(dna, expected);
    }

    #[test]
    fn test_call_no_zome() {
        let dna = test_utils::create_test_dna_with_wat("bad_zome", "test_cap", None);
        let expected = Err(HolochainError::ZomeNotFound(r#"Zome '"test_zome"' not found"#.to_string()));
        test_reduce_call(dna, expected);
    }

    #[test]
    fn test_call_ok() {
        let wasm = test_zome_api_function_wasm(ZomeAPIFunction::Call.as_str());
        let cap = create_test_cap(Membrane::Zome, &wasm);

        let dna = create_test_dna_with_cap(
            &test_zome_name(),
            "test_cap",
            &cap,
        );

        let expected = Err(HolochainError::DoesNotHaveCapabilityToken);
        test_reduce_call(dna, expected);
    }

//    #[test]
//    fn test_call_no_token() {
//        let (runtime, _) =
//            test_zome_api_function_runtime(ZomeAPIFunction::Call.as_str(), test_args_bytes());
//        assert_eq!(runtime.result, format!(r#""#),);
//    }
//
//    #[test]
//    fn test_call_no_zome() {
//        let (runtime, _) =
//            test_zome_api_function_runtime(ZomeAPIFunction::Call.as_str(), test_bad_args_bytes());
//        assert_eq!(runtime.result, format!(r#""#),);
//    }

}
