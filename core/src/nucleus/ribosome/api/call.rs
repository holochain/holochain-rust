use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::{
        is_fn_public, launch_zome_fn_call,
        ribosome::{api::ZomeApiResult, Runtime},
        state::NucleusState,
        ZomeFnCall,
    },
};
use holochain_core_types::{
    dna::{capabilities::CapabilityCall, Dna},
    entry::cap_entries::CapTokenGrant,
    error::HolochainError,
    json::JsonString,
};
use holochain_wasm_utils::api_serialization::{ZomeFnCallArgs, THIS_INSTANCE};
use jsonrpc_lite::JsonRpc;
use snowflake::ProcessUniqueId;
use std::{convert::TryFrom, sync::Arc, time::Duration};
use wasmi::{RuntimeArgs, RuntimeValue};

// ZomeFnCallArgs to ZomeFnCall
impl ZomeFnCall {
    fn from_args(args: ZomeFnCallArgs) -> Self {
        ZomeFnCall::new(&args.zome_name, args.cap, &args.fn_name, args.fn_args)
    }
}

/// HcApiFuncIndex::CALL function code
/// args: [0] encoded MemoryAllocation as u64
/// expected complex argument: {zome_name: String, cap_token: Address, fn_name: String, args: String}
/// args from API call are converted into a ZomeFnCall
/// Launch an Action::Call with newly formed ZomeFnCall
/// Waits for a ZomeFnResult
/// Returns an HcApiReturnCode as I64
pub fn invoke_call(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    println!("invoke_call 1");

    let input = match ZomeFnCallArgs::try_from(args_str.clone()) {
        Ok(input) => input,
        // Exit on error
        Err(_) => {
            runtime.context.log(format!(
                "err/zome: invoke_call failed to deserialize: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    println!("invoke_call 2");

    let result = if input.instance_handle == String::from(THIS_INSTANCE) {
        println!("invoke_call 3");
        // ZomeFnCallArgs to ZomeFnCall
        let zome_call = ZomeFnCall::from_args(input.clone());

        // Don't allow recursive calls
        if zome_call.same_fn_as(&runtime.zome_call) {
            return ribosome_error_code!(RecursiveCallForbidden);
        }
        println!("invoke_call 4");
        local_call(runtime, input)
    } else {
        println!("invoke_call 5");
        bridge_call(runtime, input)
    };
    println!("invoke_call 6");

    runtime.store_result(result)
}

fn local_call(runtime: &mut Runtime, input: ZomeFnCallArgs) -> Result<JsonString, HolochainError> {
    // ZomeFnCallArgs to ZomeFnCall
    println!("local_call 1");
    let zome_call = ZomeFnCall::from_args(input);
    println!("local_call 2");
    // Create Call Action
    let action_wrapper = ActionWrapper::new(Action::Call(zome_call.clone()));
    println!("local_call 3");

    let tick_rx = runtime.context.create_observer();
    println!("local_call 4");
    crate::instance::dispatch_action(runtime.context.action_channel(), action_wrapper);
    println!("local_call 5");

    loop {
        println!("local_call loop");
        if let Some(result) = runtime
            .context
            .state()
            .unwrap()
            .nucleus()
            .zome_call_result(&zome_call)
        {
            println!("local_call result");
            return result;
        } else {
            println!("local_call wait");
            let _ = tick_rx.recv_timeout(Duration::from_millis(10));
        }
    }
}

fn bridge_call(runtime: &mut Runtime, input: ZomeFnCallArgs) -> Result<JsonString, HolochainError> {
    let container_api =
        runtime
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

pub fn validate_call(
    context: Arc<Context>,
    state: &NucleusState,
    fn_call: &ZomeFnCall,
) -> Result<Dna, HolochainError> {
    if state.dna.is_none() {
        return Err(HolochainError::DnaMissing);
    }
    let dna = state.dna.clone().unwrap();
    // make sure the zome and function exists
    let _ = dna
        .get_function_with_zome_name(&fn_call.zome_name, &fn_call.fn_name)
        .map_err(|e| HolochainError::Dna(e))?;

    let public = is_fn_public(&dna, &fn_call)?;
    if !public && !check_capability(context.clone(), &fn_call.clone()) {
        return Err(HolochainError::CapabilityCheckFailed);
    }
    Ok(dna)
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

    // 1. Validate the call (a number of things could go wrong)
    let dna = match validate_call(context.clone(), state, &fn_call) {
        Err(err) => {
            // Notify failure
            state.zome_calls.insert(fn_call.clone(), Some(Err(err)));
            return;
        }
        Ok(dna) => dna,
    };

    // 2. Get the exposed Zome function WASM and execute it in a separate thread
    let maybe_code = dna.get_wasm_from_zome_name(fn_call.zome_name.clone());
    let code =
        maybe_code.expect("zome not found, Should have failed before when validating the call.");
    state.zome_calls.insert(fn_call.clone(), None);
    launch_zome_fn_call(context, fn_call, &code, state.dna.clone().unwrap().name);
}

// TODO: check the signature too
fn is_token_the_agent(context: Arc<Context>, cap: &Option<CapabilityCall>) -> bool {
    match cap {
        None => false,
        Some(call) => context.agent_id.key == call.cap_token.to_string(),
    }
}

/// checks to see if a given function call is allowable according to the capabilities
/// that have been registered to callers in the chain.
fn check_capability(context: Arc<Context>, fn_call: &ZomeFnCall) -> bool {
    // the agent can always do everything
    if is_token_the_agent(context.clone(), &fn_call.cap) {
        return true;
    }

    match fn_call.cap.clone() {
        None => false,
        Some(call) => {
            let chain = &context.chain_storage;
            let maybe_json = chain.read().unwrap().fetch(&call.cap_token).unwrap();
            let grant = match maybe_json {
                Some(content) => CapTokenGrant::try_from(content).unwrap(),
                None => return false,
            };
            grant.verify(call.cap_token.clone(), call.caller, &call.signature)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    extern crate test_utils;
    extern crate wabt;

    use crate::{
        context::Context,
        instance::{tests::test_instance_and_context, Instance},
        nucleus::{
            ribosome::{
                api::{
                    call::{Action, ActionWrapper, ZomeFnCall},
                    tests::{
                        test_function_name, test_parameters, test_zome_api_function_wasm,
                        test_zome_name,
                    },
                    ZomeApiFunction,
                },
                Defn,
            },
            tests::{test_capability_call, test_capability_name},
        },
        workflows::author_entry::author_entry,
    };
    use holochain_core_types::{
        cas::content::Address,
        dna::{
            capabilities::{Capability, CapabilityCall, CapabilityType},
            fn_declarations::FnDeclaration,
            Dna,
        },
        entry::Entry,
        error::{DnaError, HolochainError},
        json::JsonString,
    };
    use holochain_wasm_utils::api_serialization::ZomeFnCallArgs;

    use serde_json;
    use std::{
        collections::BTreeMap,
        sync::{
            mpsc::{sync_channel, RecvTimeoutError},
            Arc,
        },
        thread,
        time::Duration,
    };
    use test_utils::create_test_dna_with_defs;

    /// dummy commit args from standard test entry
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_bad_args_bytes() -> Vec<u8> {
        let args = ZomeFnCallArgs {
            instance_handle: "instance_handle".to_string(),
            zome_name: "zome_name".to_string(),
            cap: Some(CapabilityCall::new(Address::from("bad cap_token"), None)),
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
            cap: Some(test_capability_call()),
            fn_name: test_function_name(),
            fn_args: test_parameters(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    struct TestSetup {
        context: Arc<Context>,
        instance: Instance,
    }

    fn setup_test(dna: Dna) -> TestSetup {
        let (instance, context) =
            test_instance_and_context(dna, None).expect("Could not initialize test instance");
        TestSetup {
            context: context,
            instance: instance,
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn test_reduce_call(
        test_setup: &TestSetup,
        token_str: &str,
        _caller: Address,
        expected: Result<Result<JsonString, HolochainError>, RecvTimeoutError>,
    ) {
        let zome_call = ZomeFnCall::new(
            "test_zome",
            Some(CapabilityCall::new(Address::from(token_str), None)),
            "test",
            "{}",
        );
        let zome_call_action = ActionWrapper::new(Action::Call(zome_call.clone()));

        let (_, rx_observer) = sync_channel(1);
        test_setup.instance.process_action(
            zome_call_action,
            Vec::new(),
            &rx_observer,
            &test_setup.context,
        );

        while test_setup
            .instance
            .state()
            .nucleus()
            .zome_call_result(&zome_call)
            .is_none()
        {
            thread::sleep(Duration::from_millis(10));
        }

        let action_result = Ok(test_setup
            .instance
            .state()
            .nucleus()
            .zome_call_result(&zome_call)
            .unwrap());

        assert_eq!(expected, action_result);
    }

    #[test]
    fn test_call_no_zome() {
        let dna = test_utils::create_test_dna_with_wat("bad_zome", &test_capability_name(), None);
        let test_setup = setup_test(dna);
        let expected = Ok(Err(HolochainError::Dna(DnaError::ZomeNotFound(
            r#"Zome 'test_zome' not found"#.to_string(),
        ))));
        test_reduce_call(&test_setup, "foo token", Address::from("caller"), expected);
    }

    fn setup_dna_for_cap_test(cap_type: CapabilityType) -> Dna {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::Call.as_str());
        let mut capability = Capability::new(cap_type);
        let fn_decl = FnDeclaration {
            name: test_function_name(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        };
        capability.functions = vec![fn_decl.name.clone()];
        let mut capabilities = BTreeMap::new();
        capabilities.insert(test_capability_name(), capability);
        let mut functions = Vec::new();
        functions.push(fn_decl);

        create_test_dna_with_defs(&test_zome_name(), (functions, capabilities), &wasm)
    }

    #[test]
    fn test_call_public() {
        let dna = setup_dna_for_cap_test(CapabilityType::Public);
        let test_setup = setup_test(dna);
        // Expecting error since there is no function in wasm to call
        let expected = Ok(Err(HolochainError::RibosomeFailed(
            "Argument deserialization failed".to_string(),
        )));
        test_reduce_call(&test_setup, "", Address::from("caller"), expected);
    }

    #[test]
    fn test_call_transferable() {
        let dna = setup_dna_for_cap_test(CapabilityType::Transferable);
        let test_setup = setup_test(dna);
        let expected_failure = Ok(Err(HolochainError::CapabilityCheckFailed));
        test_reduce_call(&test_setup, "", Address::from("caller"), expected_failure);

        // Expecting error since there is no function in wasm to call
        let expected = Ok(Err(HolochainError::RibosomeFailed(
            "Argument deserialization failed".to_string(),
        )));
        let agent_token_str = test_setup.context.agent_id.key.clone();
        test_reduce_call(
            &test_setup,
            &agent_token_str,
            Address::from(agent_token_str.clone()),
            expected.clone(),
        );

        let grant = CapTokenGrant::create(CapabilityType::Transferable, None).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let addr = test_setup
            .context
            .block_on(author_entry(&grant_entry, None, &test_setup.context))
            .unwrap();
        test_reduce_call(
            &test_setup,
            &String::from(addr),
            Address::from("any caller"),
            expected,
        );
    }

    #[test]
    fn test_call_assigned() {
        let dna = setup_dna_for_cap_test(CapabilityType::Assigned);
        let test_setup = setup_test(dna);
        let expected_failure = Ok(Err(HolochainError::CapabilityCheckFailed));
        test_reduce_call(
            &test_setup,
            "",
            Address::from("any caller"),
            expected_failure.clone(),
        );

        // Expecting error since there is no function in wasm to call
        let expected = Ok(Err(HolochainError::RibosomeFailed(
            "Argument deserialization failed".to_string(),
        )));
        let agent_token_str = test_setup.context.agent_id.key.clone();
        test_reduce_call(
            &test_setup,
            &agent_token_str,
            Address::from(agent_token_str.clone()),
            expected.clone(),
        );

        let someone = Address::from("somoeone");
        let grant =
            CapTokenGrant::create(CapabilityType::Assigned, Some(vec![someone.clone()])).unwrap();
        let grant_entry = Entry::CapTokenGrant(grant);
        let addr = test_setup
            .context
            .block_on(author_entry(&grant_entry, None, &test_setup.context))
            .unwrap();
        test_reduce_call(
            &test_setup,
            &String::from(addr.clone()),
            someone,
            expected.clone(),
        );

        /* function call doesn't know who the caller is yet so can't do the check in reduce
                let someone_else = Address::from("somoeone_else");
                test_reduce_call(&test_setup,&String::from(addr),someone_else, expected_failure.clone());
        */
    }

    #[test]
    fn test_agent_as_token() {
        let dna = test_utils::create_test_dna_with_wat("bad_zome", "test_cap", None);
        let test_setup = setup_test(dna);
        let agent_token = Address::from(test_setup.context.agent_id.key.clone());
        let context = test_setup.context.clone();
        let cap_call = CapabilityCall::new(agent_token, None);
        assert!(is_token_the_agent(context.clone(), &Some(cap_call)));
        let cap_call = CapabilityCall::new(Address::from(""), None);
        assert!(!is_token_the_agent(context, &Some(cap_call)));
    }
}
